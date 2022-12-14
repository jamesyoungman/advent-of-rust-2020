#[macro_use]
extern crate lazy_static;
extern crate log;
extern crate pretty_env_logger;
extern crate regex;

use std::collections::HashSet;
use std::collections::VecDeque;
use std::fmt;
use std::io;
use std::io::BufRead;

use regex::Regex;

lazy_static! {
    static ref PLAYER_RE: Regex = Regex::new("^Player ([12]):$").unwrap();
}

type Card = u32;

fn get_deck(who: i32, lines: &[String]) -> Result<VecDeque<Card>, String> {
    let mut result: VecDeque<Card> = VecDeque::new();
    let mut player = 0;
    for line in lines.iter().filter(|s| !s.is_empty()) {
        match PLAYER_RE.captures(line) {
            None => (),
            Some(cap) => match cap[1].parse() {
                Ok(n) => {
                    player = n;
                    continue;
                }
                Err(e) => {
                    return Err(format!("cannot parse '{}' as an integer: {}", &cap[1], e));
                }
            },
        }
        if player == who {
            match line.parse() {
                Ok(n) => result.push_back(n),
                Err(e) => {
                    return Err(format!("cannot parse '{}' as an integer: {}", line, e));
                }
            }
        }
    }
    Ok(result)
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash)]
struct PlayerHand {
    cards: VecDeque<Card>,
}

impl Clone for PlayerHand {
    fn clone(&self) -> PlayerHand {
        PlayerHand {
            cards: self.cards.clone(),
        }
    }
}

impl PlayerHand {
    fn new(who: i32, config_lines: &[String]) -> Result<PlayerHand, String> {
        let cards = get_deck(who, config_lines)?;
        Ok(PlayerHand { cards })
    }

    fn score(&self) -> usize {
        let total = self
            .cards
            .iter()
            .rev()
            .enumerate()
            .map(|(i, c)| (i + 1) * (*c) as usize)
            .sum();
        total
    }

    fn play_next_card(&mut self) -> Card {
        self.cards.pop_front().expect("hand should not be empty")
    }

    fn win_cards(&mut self, first: Card, second: Card) {
        self.cards.push_back(first);
        self.cards.push_back(second);
    }

    fn truncate(&mut self, max_cards: usize) {
        self.cards.truncate(max_cards);
    }

    fn num_cards(&self) -> usize {
        self.cards.len()
    }

    fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

impl fmt::Display for PlayerHand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&itertools::join(
            self.cards.iter().map(|c| format!("{}", c)),
            ", ",
        ))
    }
}

#[derive(PartialEq, Eq, Hash)]
struct GameState {
    p1: PlayerHand,
    p2: PlayerHand,
}

impl Clone for GameState {
    fn clone(&self) -> GameState {
        GameState {
            p1: self.p1.clone(),
            p2: self.p2.clone(),
        }
    }
}

impl GameState {
    fn new(lines: &[String]) -> Result<GameState, String> {
        let p1 = PlayerHand::new(1, lines)?;
        let p2 = PlayerHand::new(2, lines)?;
        Ok(GameState { p1, p2 })
    }

    fn decks_as_string(&self) -> String {
        format!("Player 1: {}\nPlayer 2: {}", self.p1, self.p2)
    }

    fn truncate_hands(&mut self, p1max: usize, p2max: usize) {
        self.p1.truncate(p1max);
        self.p2.truncate(p2max);
    }

    fn total_card_count(&self) -> usize {
        self.p1.num_cards() + self.p2.num_cards()
    }

    fn score(&self, who: i32) -> usize {
        match who {
            1 => self.p1.score(),
            2 => self.p2.score(),
            _ => panic!("There is no player {}", who),
        }
    }

    fn game_result(&self) -> Option<(i32, &PlayerHand, usize)> {
        match (self.p1.is_empty(), self.p2.is_empty()) {
            (true, false) => Some((2, &self.p2, self.p2.score())),
            (false, true) => Some((1, &self.p1, self.p1.score())),
            (false, false) => None,
            (true, true) => {
                // In the game loop we assert that the total card
                // count is invariant, so that assertion should fire
                // before we get here.
                panic!("impossible");
            }
        }
    }

    fn play_one_basic_round(&mut self, round: i32) -> bool {
        log::info!("\n--Round {} --\n", round);
        if log::log_enabled!(log::Level::Debug) {
            for s in self.decks_as_string().split('\n') {
                log::debug!("{}", s);
            }
        }

        let (c1, c2) = (self.p1.play_next_card(), self.p2.play_next_card());
        if c2 > c1 {
            self.p2.win_cards(c2, c1);
        } else {
            self.p1.win_cards(c1, c2);
        }
        self.p1.is_empty() || self.p2.is_empty()
    }

    fn play_basic(&mut self) -> (i32, &PlayerHand, usize) {
        let ncards = self.total_card_count();
        for round in 1.. {
            assert!(ncards == self.total_card_count());
            if self.play_one_basic_round(round) {
                break; // someone won
            }
        }
        if log::log_enabled!(log::Level::Info) {
            log::info!("\n== Post-game results ==\n{}", self.decks_as_string());
        }
        match self.game_result() {
            Some(r) => r,
            None => {
                panic!("terminated play_basic before game was over");
            }
        }
    }

    fn play_recursive(&mut self, game_counter: &mut usize) -> (i32, &PlayerHand, usize) {
        let mut previous_states: HashSet<GameState> = HashSet::new();
        let this_game = *game_counter;
        log::debug!("== Game {} ===", this_game);
        let ncards = self.total_card_count();
        for round in 1.. {
            log::debug!("");
            log::debug!("-- Round {} (Game {}) --", round, this_game);
            if log::log_enabled!(log::Level::Info) {
                for s in self.decks_as_string().split('\n') {
                    log::info!("{}", s);
                }
            }
            assert!(self.total_card_count() == ncards);
            if previous_states.contains(self) {
                // player 1 wins the game.
                return (1, &self.p1, self.score(1));
            }
            previous_states.insert(self.clone());

            let c1 = self.p1.play_next_card();
            log::debug!("Player 1 plays: {}", c1);
            let c2 = self.p2.play_next_card();
            log::debug!("Player 2 plays: {}", c2);
            //assert!(c1 >= 0 && c2 >= 0);
            let recurse =
                (c1 as usize) <= self.p1.num_cards() && (c2 as usize) <= self.p2.num_cards();
            let winner = if recurse {
                let mut subgame = self.clone();
                subgame.truncate_hands(c1 as usize, c2 as usize);
                *game_counter += 1;
                log::debug!("Playing a sub-game to determine the winner...");
                let (winner, _, _) = subgame.play_recursive(game_counter);
                winner
            } else if c2 > c1 {
                2
            } else {
                1
            };
            log::debug!(
                "Player {} wins round {} of game {}!",
                winner,
                round,
                this_game
            );
            match winner {
                1 => self.p1.win_cards(c1, c2),
                2 => self.p2.win_cards(c2, c1),
                _ => {
                    panic!("unexpectedly, player N enters the game!");
                }
            }
            if self.p1.is_empty() || self.p2.is_empty() {
                let (winner, hand, score) = self.game_result().expect("someone won");
                log::info!("The winner of game {} is player {}", this_game, winner);
                return (winner, hand, score);
            }
        }
        panic!("did not expect to fall out of the bottom of the loop");
    }
}

fn part1(lines: &[String]) -> Result<(), String> {
    let mut game_state = GameState::new(lines)?;
    let (winner, hand, score) = game_state.play_basic();
    println!(
        "Part 1: winner is player {} with hand {}; score is {}",
        winner, hand, score
    );
    Ok(())
}

fn part2(lines: &[String]) -> Result<(), String> {
    let mut game_counter: usize = 1;
    let mut game_state = GameState::new(lines)?;
    let (winner, _, score) = game_state.play_recursive(&mut game_counter);
    log::info!("== Post-game results ==\n{}", &game_state.decks_as_string());
    println!("Part 2: winner is player {}; score is {}", winner, score);
    Ok(())
}

fn run() -> Result<(), String> {
    let mut lines: Vec<String> = Vec::new();
    for line_or_err in io::BufReader::new(io::stdin()).lines() {
        match line_or_err {
            Err(e) => {
                return Err(format!("I/O error: {}", e));
            }
            Ok(line) => {
                lines.push(line);
            }
        }
    }
    part1(&lines)?;
    part2(&lines)?;
    Ok(())
}

fn main() {
    // the env logger is configured with $RUST_LOG.
    // For example RUST_LOG=debug day22
    pretty_env_logger::init();
    std::process::exit(match run() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("error: {}", err);
            1
        }
    });
}
