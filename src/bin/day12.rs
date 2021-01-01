extern crate itertools;

use std::io::BufRead;
use std::string::String;
use std::io;

static DIRECTIONS: &'static [(i64, i64, &str)] = &[(1,0,"East"),
						   (0,-1, "South"),
						   (-1,0, "West"),
						   (0,1, "North")];

fn normalise_heading(mut h: i64) -> i64 {
    let n = DIRECTIONS.len() as i64;
    while h >= n {
        h -= n;
    }
    while h < 0 {
        h += n;
    }
    h
}


fn read_input(reader: impl BufRead) -> Result<Vec<String>, String> {
    match reader.lines().collect() {
	Err(e) => Err(format!("I/O error: {}", e)),
	Ok(r) => Ok(r),
    }
}

fn parse_instr(s: &str) -> Result<(String, i64), String> {
    let instruction = &s[0..1];
    match s[1..].parse() {
	Ok(n) => Ok((instruction.to_string(), n)),
	Err(e) => Err(format!("failed to parse '{}': {}", s, e))
    }
}

fn rotate(direction: &str,
	  mut amount: i64,
	  mut waypoint_rel_x: i64,
	  mut waypoint_rel_y: i64) -> (i64, i64) {
    amount = match direction {
	"L" => 360 - amount,
	"R" => amount,
	_ => {
	    panic!("bad direction");
	}
    };
    amount = match amount {
	0 | 90 | 180 | 270 => amount / 90,
	_ => panic!("unsupported rotation amount")
    };
    while amount > 0 {
	let t = waypoint_rel_x;
	waypoint_rel_x = waypoint_rel_y;
	waypoint_rel_y = -t;
	amount -= 1
    }
    (waypoint_rel_x, waypoint_rel_y)
}

fn part2(instructions: &Vec<String>) -> Result<(), String> {
    let mut heading: i64 = 0;
    let mut ship_x: i64 = 0;
    let mut ship_y: i64 = 0;
    let mut waypoint_rel_x: i64 = 10;
    let mut waypoint_rel_y: i64 = 1;

    for line in instructions {
	let (instruction, amount) = parse_instr(line)?;
	match instruction.as_str() {
	    "N" => { waypoint_rel_y += amount }
	    "E" => { waypoint_rel_x += amount }
	    "S" => { waypoint_rel_y -= amount }
	    "W" => { waypoint_rel_x -= amount }
	    "R"|"L" => {
		let rotated = rotate(instruction.as_str(), amount,
				     waypoint_rel_x, waypoint_rel_y);
		waypoint_rel_x = rotated.0;
		waypoint_rel_y = rotated.1;
	    }
	    "F" => {
		ship_x += waypoint_rel_x * amount;
		ship_y += waypoint_rel_y * amount;
	    }
	    _ => {
		return Err(format!("unknown instruction: {}", instruction));
	    }
	}
	heading = normalise_heading(heading);
	println!(
	    "After instruction {:>6}, ship position=({:>6},{:>6}), waypoint=({:>3},{:3>})",
	    line, ship_x, ship_y, waypoint_rel_x, waypoint_rel_y);
    }
    println!("Part 2: manhattan distance {}", (ship_x.abs() + ship_y.abs()));
    Ok(())
}


fn part1(instructions: &Vec<String>) -> Result<(), String> {
    let mut heading: i64 = 0;
    let mut x: i64 = 0;
    let mut y: i64 = 0;
    for line in instructions {
	let (instruction, amount) = parse_instr(line)?;
	match instruction.as_str() {
	    "N" => { y += amount }
	    "E" => { x += amount }
	    "S" => { y -= amount }
	    "W" => { x -= amount }
	    "R" => { heading += amount/90 }
	    "L" => { heading -= amount/90 }
	    "F" => {
		assert!(heading >= 0);
		x += DIRECTIONS[heading as usize].0 * amount;
		y += DIRECTIONS[heading as usize].1 * amount;
	    }
	    _ => {
		return Err(format!("unknown instruction: {}", instruction));
	    }
	}
	heading = normalise_heading(heading);
	println!("After instruction {:>6}, position=({:>6},{:>6}), heading={:<5}",
		 line, x, y, DIRECTIONS[heading as usize].2);
    }
    println!("Part 1: manhattan distance {}", (x.abs() + y.abs()));
    Ok(())
}

fn run() -> Result<(), String> {
    let instructions = read_input(io::BufReader::new(io::stdin()))?;
    part1(&instructions)?;
    part2(&instructions)?;
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
