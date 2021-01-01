extern crate itertools;

use std::fmt;
use std::io::BufRead;
use std::string::String;
use std::io;


#[derive(Clone, Copy, Debug)]
enum Position {
    Seat(bool),
    Floor
}
static FLOOR: &str = ".";
static OCCUPIED: &str = "#";
static EMPTY: &str = "L";


fn pos_as_str(p: &Position) -> &'static str {
    match p {
	Position::Seat(true) => OCCUPIED,
	Position::Seat(false) => EMPTY,
	Position::Floor => FLOOR,
    }
}

fn seat_occupation(p: &Position) -> usize {
    match p {
	Position::Seat(true) => 1,
	_ => 0,
    }
}

#[derive(Debug)]
struct Direction {
    dx: i64,
    dy: i64,
}

static ALL_DIRECTIONS: [Direction; 8] = [
    Direction{dx: 0, dy: -1},	// N
    Direction{dx: 1, dy: -1},	// NE
    Direction{dx: 1, dy: 0},	// E
    Direction{dx: 1, dy: 1},	// SE
    Direction{dx: 0, dy: 1},	// S
    Direction{dx: -1, dy: 1},	// SW
    Direction{dx: -1, dy: 0},	// W
    Direction{dx: -1, dy: -1},	// NW
];


struct Grid {
    seats: Vec<Vec<Position>>,
    grid_width: usize,

}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
	f.write_str(
	    &itertools::join(
		self.seats.iter().flat_map(|row|
					   itertools::chain(
					       row.iter().map(pos_as_str),
					       std::iter::once("\n"))),
		""))
    }
}

impl Clone for Grid {
    fn clone(&self) -> Self {
	Grid{
	    seats: self.seats.clone(),
	    grid_width: self.grid_width,
	}
    }
}

fn get_next(current: &Position,
	    neighbour_count: usize,
	    overcrowding_limit: usize) -> (Position, bool) {
    match (current, neighbour_count) {
	// An empty seat becomes occupied if there are no
	// occupied neighbour seats
	(Position::Seat(false), 0) => {
	    (Position::Seat(true), true)
	}
	// An occupied seat becomes empty if there are too many
	// occupied neighbour seats
	(Position::Seat(true), n) if n >= overcrowding_limit => {
	    (Position::Seat(false), true)
	}
	(Position::Floor, _) => (Position::Floor, false),
	// Otherwise the seat is unchanged.
	_ => (*current, false),
    }
}

impl Grid {
    fn new(lines: &Vec<String>) -> Result<Grid, String> {
	let height = lines.len();
	if height == 0 {
	    return Ok(Grid{
		seats: Vec::new(),
		grid_width: 0,
	    }); // zero-sized.
	}
	let maxwidth = lines.iter().map(|line| line.len()).max().unwrap();
	let minwidth = lines.iter().map(|line| line.len()).min().unwrap();
	if maxwidth != minwidth {
	    return Err(format!("Variable length lines ({} versus {})",
			       minwidth, maxwidth));
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
	Ok(Grid{
	    seats: sd,
	    grid_width: maxwidth,
	})
    }

    fn height(&self) -> usize {
	self.seats.len()
    }

    fn total_occupation(&self) -> usize {
	fn row_occupation(row: &Vec<Position>) -> usize {
	    row.iter().map(seat_occupation).sum()
	}
	self.seats.iter().map(row_occupation).sum()
    }

    fn at(&self, x: i64, y: i64) -> Option<&Position> {
	if x < 0 || y < 0 {
	    None
	} else if (x as usize) >= self.grid_width {
	    None
	} else if (y as usize) >= self.seats.len() {
	    None
	} else {
	    Some(&self.seats[y as usize][x as usize])
	}
    }

    fn immediate_neighbours_occupied(&self, x: i64, y: i64) -> usize {
	ALL_DIRECTIONS.iter()
	   .map(|d|
		match self.at(x + d.dx, y + d.dy) {
		    Some(Position::Seat(true)) => 1,
		    _ => 0,
		})
	   .sum()
    }

    fn line_of_sight_neighbour(&self, x: i64, y: i64, d: &Direction) -> usize {
	let mut nx = x;
	let mut ny = y;
	for _dist in 1.. {
	    nx += d.dx;
	    ny += d.dy;
	    match self.at(nx, ny) {
		None => {
		    return 0;
		}
		Some(Position::Floor) => {
		    continue;
		}
		Some(Position::Seat(true)) => {
		    return 1;
		}
		Some(Position::Seat(false)) => {
		    return 0;
		}
	    }
	}
	panic!("an infinite loop terminated");
    }

    fn line_of_sight_neighbours_occupied(&self, x: i64, y: i64) -> usize {
	let mut total: usize = 0;
	for d in &ALL_DIRECTIONS {
	    let occ: usize = self.line_of_sight_neighbour(x, y, d);
	    total += occ;
	}
	total
    }

    fn iterate<OccCounter>(&self,
	       neighbour_counter: OccCounter,
	       overcrowding_limit: usize) -> Result<(Grid, bool), String>
    where OccCounter: Fn(&Grid, i64, i64) -> usize {
	let mut changed: bool = false;
	let mut next: Vec<Vec<Position>> = Vec::new();
	next.resize_with(self.height(), Vec::new);
	for (y, row) in self.seats.iter().enumerate() {
	    next[y].resize(self.height(), Position::Floor);
	    for (x, current) in row.iter().enumerate() {
		let nc: usize = neighbour_counter(self, x as i64, y as i64);
		match get_next(current, nc, overcrowding_limit) {
		    (p, change) => {
			if change {
			    changed = true;
			}
			next[y][x] = p;
		    }
		}
	    }
	}
	Ok((Grid{
	    seats: next,
	    grid_width: self.grid_width,

	}, changed))
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

fn iterate_until_stable<OccCounter>(initial: &Grid,
				    occ_counter: &OccCounter,
				    overcrowding_limit: usize) -> Result<(i64, Grid), String>
where OccCounter: Fn(&Grid, i64, i64) -> usize {
    let mut current: Grid = initial.clone();
    for iteration in 1.. {
	//println!("iterate_until_stable: iteration {}:\n{}",
	//	 iteration, current);
	match current.iterate(occ_counter, overcrowding_limit) {
	    Ok((next, true)) => {
		current = next
	    }
	    Ok((_, false)) => {
		println!("Stable at iteration {}:\n{}", iteration, current);
		return Ok((iteration, current));
	    }
	    Err(e) => {
		return Err(e);
	    }
	}
    }
    Err("an infinite loop ended".to_string())
}


fn solve<OccCounter>(description: &str,
		     initial: &Grid,
		     occ_counter: &OccCounter,
		     overcrowding_limit: usize) -> Result<(), String>
where OccCounter: Fn(&Grid, i64, i64)-> usize {
    println!("{}: initial state:\n{}", description, initial);
    println!("{}: initial seat occupation is {}", description, initial.total_occupation());
    let end_state: Result<(i64, Grid), String> = iterate_until_stable(
	initial, occ_counter, overcrowding_limit);
    match end_state {
	Ok((iterations, final_grid)) => {
	    println!("{}: stable after {} iterations:\n{}\n{} seats are occupied.",
		     description, iterations, final_grid, final_grid.total_occupation());
	    Ok(())
	}
	Err(e) => Err(format!("{} failed: {}", description, e))
    }
}

fn run() -> Result<(), String> {
    let initial = read_input(io::BufReader::new(io::stdin()))?;
    solve("Part 1", &initial, &Grid::immediate_neighbours_occupied, 4)?;
    solve("Part 2", &initial, &Grid::line_of_sight_neighbours_occupied, 5)?;
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
