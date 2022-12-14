extern crate itertools;

use std::fmt;
use std::io;
use std::io::BufRead;
use std::string::String;

#[derive(Clone, Copy, Debug)]
enum Position {
    Seat(bool),
    Floor,
}
static FLOOR: &str = ".";
static OCCUPIED: &str = "#";
static EMPTY: &str = "L";

#[derive(Debug)]
struct Direction {
    dx: i64,
    dy: i64,
}

static ALL_DIRECTIONS: [Direction; 8] = [
    Direction { dx: 0, dy: -1 },  // N
    Direction { dx: 1, dy: -1 },  // NE
    Direction { dx: 1, dy: 0 },   // E
    Direction { dx: 1, dy: 1 },   // SE
    Direction { dx: 0, dy: 1 },   // S
    Direction { dx: -1, dy: 1 },  // SW
    Direction { dx: -1, dy: 0 },  // W
    Direction { dx: -1, dy: -1 }, // NW
];

struct Grid {
    seats: Vec<Vec<Position>>,
    grid_width: usize,
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn pos_as_str(p: &Position) -> &'static str {
            match p {
                Position::Seat(true) => OCCUPIED,
                Position::Seat(false) => EMPTY,
                Position::Floor => FLOOR,
            }
        }

        f.write_str(&itertools::join(
            self.seats.iter().flat_map(|row| {
                itertools::chain(row.iter().map(pos_as_str), std::iter::once("\n"))
            }),
            "",
        ))
    }
}

impl Clone for Grid {
    fn clone(&self) -> Self {
        Grid {
            seats: self.seats.clone(),
            grid_width: self.grid_width,
        }
    }
}

fn get_next(
    current: &Position,
    neighbour_count: usize,
    overcrowding_limit: usize,
) -> (Position, bool) {
    match (current, neighbour_count) {
        // An empty seat becomes occupied if there are no
        // occupied neighbour seats
        (Position::Seat(false), 0) => (Position::Seat(true), true),
        // An occupied seat becomes empty if there are too many
        // occupied neighbour seats
        (Position::Seat(true), n) if n >= overcrowding_limit => (Position::Seat(false), true),
        (Position::Floor, _) => (Position::Floor, false),
        // Otherwise the seat is unchanged.
        _ => (*current, false),
    }
}

impl Grid {
    fn new(lines: &Vec<String>) -> Result<Grid, String> {
        let height = lines.len();
        if height == 0 {
            return Ok(Grid {
                seats: Vec::new(),
                grid_width: 0,
            }); // zero-sized.
        }
        let maxwidth = lines.iter().map(|line| line.len()).max().unwrap();
        let minwidth = lines.iter().map(|line| line.len()).min().unwrap();
        if maxwidth != minwidth {
            return Err(format!(
                "Variable length lines ({} versus {})",
                minwidth, maxwidth
            ));
        }
        let mut sd = Vec::new();
        sd.resize_with(height, Vec::new);
        for (y, line) in lines.iter().enumerate() {
            sd[y].resize(maxwidth, Position::Floor);
            for (x, ch) in line.chars().enumerate() {
                sd[y][x] = match ch {
                    '#' => Position::Seat(true),
                    'L' => Position::Seat(false),
                    '.' => Position::Floor,
                    _ => {
                        return Err(format!("unexpected input character '{}'", ch));
                    }
                }
            }
        }
        Ok(Grid {
            seats: sd,
            grid_width: maxwidth,
        })
    }

    fn height(&self) -> usize {
        self.seats.len()
    }

    fn total_occupation(&self) -> usize {
        fn row_occupation(row: &[Position]) -> usize {
            row.iter()
                .map(|p| match p {
                    Position::Seat(true) => 1,
                    _ => 0,
                })
                .sum()
        }
        self.seats.iter().map(|s| row_occupation(s)).sum()
    }

    fn at(&self, x: i64, y: i64) -> Option<&Position> {
        if x < 0 || y < 0 || (x as usize) >= self.grid_width || (y as usize) >= self.seats.len() {
            None
        } else {
            Some(&self.seats[y as usize][x as usize])
        }
    }

    fn immediate_neighbours_occupied(&self, x: i64, y: i64) -> usize {
        ALL_DIRECTIONS
            .iter()
            .map(|d| match self.at(x + d.dx, y + d.dy) {
                Some(Position::Seat(true)) => 1,
                _ => 0,
            })
            .sum()
    }

    fn line_of_sight_neighbour(&self, x: i64, y: i64, d: &Direction) -> usize {
        for i in 1.. {
            match self.at(x + i * d.dx, y + i * d.dy) {
                None => return 0,
                Some(Position::Seat(true)) => return 1,
                Some(Position::Seat(false)) => return 0,
                Some(Position::Floor) => (), // keep going.
            }
        }
        panic!("an infinite loop terminated");
    }

    fn line_of_sight_neighbours_occupied(&self, x: i64, y: i64) -> usize {
        ALL_DIRECTIONS
            .iter()
            .map(|d| self.line_of_sight_neighbour(x, y, d))
            .sum()
    }

    fn iterate<OccCounter>(
        &self,
        neighbour_counter: OccCounter,
        overcrowding_limit: usize,
    ) -> (Grid, bool)
    where
        OccCounter: Fn(&Grid, i64, i64) -> usize,
    {
        let mut changed: bool = false;
        let mut next: Vec<Vec<Position>> = Vec::new();
        next.resize_with(self.height(), Vec::new);
        for (y, row) in self.seats.iter().enumerate() {
            next[y].resize(self.height(), Position::Floor);
            for (x, current) in row.iter().enumerate() {
                let (p, change) = get_next(
                    current,
                    neighbour_counter(self, x as i64, y as i64),
                    overcrowding_limit,
                );
                if change {
                    changed = true;
                }
                next[y][x] = p;
            }
        }
        (
            Grid {
                seats: next,
                grid_width: self.grid_width,
            },
            changed,
        )
    }
}

fn read_input(reader: impl BufRead) -> Result<Grid, String> {
    let mut lines: Vec<String> = Vec::new();
    for line_or_fail in reader.lines() {
        match line_or_fail {
            Ok(line) => {
                lines.push(line);
            }
            Err(e) => {
                return Err(format!("I/O error: {}", e));
            }
        }
    }
    Grid::new(&lines)
}

fn iterate_until_stable<OccCounter>(
    initial: &Grid,
    occ_counter: &OccCounter,
    overcrowding_limit: usize,
) -> (i64, Grid)
where
    OccCounter: Fn(&Grid, i64, i64) -> usize,
{
    let mut current: Grid = initial.clone();
    for iteration in 1.. {
        let (next, changed) = current.iterate(occ_counter, overcrowding_limit);
        if !changed {
            println!("Stable at iteration {}:\n{}", iteration, current);
            return (iteration, current);
        }
        current = next
    }
    unreachable!()
}

fn solve<OccCounter>(description: &str, initial: &Grid, occ_counter: &OccCounter, limit: usize)
where
    OccCounter: Fn(&Grid, i64, i64) -> usize,
{
    println!("{}: initial state:\n{}", description, initial);
    println!(
        "{}: initial seat occupation is {}",
        description,
        initial.total_occupation()
    );
    let (iterations, final_grid) = iterate_until_stable(initial, occ_counter, limit);
    println!(
        "Done:\n{}\n{}: stable after {} iterations; {} seats are occupied.",
        final_grid,
        description,
        iterations,
        final_grid.total_occupation()
    );
}

fn run() -> Result<(), String> {
    let initial = read_input(io::BufReader::new(io::stdin()))?;
    solve("Part 1", &initial, &Grid::immediate_neighbours_occupied, 4);
    solve(
        "Part 2",
        &initial,
        &Grid::line_of_sight_neighbours_occupied,
        5,
    );
    Ok(())
}

fn main() {
    std::process::exit(match run() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("error: {:?}", err);
            1
        }
    });
}
