extern crate log;
extern crate pretty_env_logger;
use std::collections::HashSet;
use std::io::prelude::*;
use std::io;

#[derive(Debug,PartialEq,Eq)]
enum Direction {		// note that N and S are deliberately missing.
    NE,
    E,
    SE,
    SW,
    W,
    NW,
}

impl Direction {
    // Directions here are vectors on a modified cartesian coordinate
    // plane.  The locations are hexagons.  Visualise the hexagons
    // with corners pointing North/South.  The X-axis runs riectly
    // East/West (through the centres of two of the sides).  If we
    // move NE then NW, this should leave the X co-ordinate unchanged.
    // Moving only NE, though, does change the X co-ordinate.  To
    // obtain these properties while keeping things simple, we adopt
    // the convention that moving E or W changes the X coordinate by
    // 2, not 1.
    //
    // You can visualise this as if alternate rows of hexagons have X
    // coordinates in between the X coordinates of the rows above and
    // below.
    fn delta(&self) -> (i32,i32) { // (dx, dy)
	match self {
	    Direction::NE => (1, 1),
	    Direction::E => (2, 0),
	    Direction::SE => (1, -1),
	    Direction::SW => (-1, -1),
	    Direction::W => (-2, 0),
	    Direction::NW => (-1, 1),
	}
    }
}

#[derive(Debug,PartialEq,Eq,Hash)]
struct Position {
    x: i32,
    y: i32,
}

impl Position {
    fn delta(&self, d: &Direction) -> Position {
	let delta = d.delta();
	Position{
	    x: self.x + delta.0,
	    y: self.y + delta.1,
	}
    }
}


struct Floor {
    // The default state of a tile is white, so we only record the
    // locations of the black tiles.
    black_tiles: HashSet<Position>
}

impl Floor {
    fn new() -> Floor {
	Floor{
	    black_tiles: HashSet::new(),
	}
    }

    fn flip(&mut self, p: Position) {
	if self.black_tiles.contains(&p) {
	    self.black_tiles.remove(&p);
	} else {
	    self.black_tiles.insert(p);
	}
    }

    fn count_black_tiles(&self) -> usize {
	self.black_tiles.len()
    }

    fn obey(&mut self, line: &str) {
	let final_pos = split_directions(line).iter()
	    .fold(Position{x: 0, y: 0}, |p, d| p.delta(d));
	self.flip(final_pos)
    }
}

fn get_direction(saved: &Option<char>, current: char) -> (Option<char>, Option<Direction>) {
    match (saved, current) {
	// Note that there is no "North" or "South" output, so both
	// 'n' and 's' will always be followed by 'e' or 'w'.
	(None, 'n') => (Some(current), None), // next letter is 'e' or 'w'
	(None, 's') => (Some(current), None), // next letter is 'e' or 'w'
	(Some('n'), 'e') => (None, Some(Direction::NE)),
	(None, 'e') => (None, Some(Direction::E)),
	(Some('s'), 'e') => (None, Some(Direction::SE)),
	(Some('s'), 'w') => (None, Some(Direction::SW)),
	(None, 'w') => (None, Some(Direction::W)),
	(Some('n'), 'w') => (None, Some(Direction::NW)),
	_ => {
	    panic!(format!("get_direction: invalid state, saved={:?}, current={:?}",
			   saved, current));
	}
    }
}

fn split_directions(s: &str) -> Vec<Direction> {
    let mut result = Vec::new();
    let mut saved: Option<char> = None;
    for ch in s.chars() {
	let (s, d) = get_direction(&saved, ch);
	match d {
	    None => (),
	    Some(dir) => result.push(dir),
	}
	saved = s;
    }
    result
}


fn solve1(lines: &Vec<String>) -> Floor {
    let mut floor: Floor = Floor::new();
    for line in lines {
	floor.obey(line);
    }
    floor
}




fn part1(lines: &Vec<String>) -> Result<(), String> {
    println!("Part 1: number of black-side-up tiles: {}",
	     solve1(lines).count_black_tiles());
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
    //part2(&lines)?;
    Ok(())
}

fn main() {
    // the env logger is configured with $RUST_LOG.
    // For example RUST_LOG=debug day24
    pretty_env_logger::init();
    std::process::exit(match run() {
	Ok(_) => 0,
	Err(err) => {
	    eprintln!("error: {}", err);
	    1
	}
    });
}
