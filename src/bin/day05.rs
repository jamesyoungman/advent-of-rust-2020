extern crate thiserror;
use std::io;
use std::io::BufRead;
use std::collections::BTreeSet;
use thiserror::Error;

#[derive(Error,Debug)]
pub enum SeatError {
    #[error("empty starting range")]
    EmptyStartingRange,
    #[error("input is empty")]
    NoInput,
    #[error("input is invalid; {0}")]
    InvalidInput(String),
    #[error("Read error")]
    ReadError { source: std::io::Error },
}

fn binary_search(mut lower_incl: i32, mut upper_excl: i32,
	     lower_directive: char, upper_directive: char,
	     choices: &str) -> Result<i32, SeatError> {
    if lower_incl >= upper_excl {
	return Err(SeatError::EmptyStartingRange);
    }
    for choice in choices.chars() {
	if upper_excl == lower_incl + 1 {
	    return Err(SeatError::InvalidInput(
		"too many seat directions".to_string()));
	}
	let mid = lower_incl + (upper_excl - lower_incl)/2;
	if choice == lower_directive {
	    upper_excl = mid
	} else if choice == upper_directive {
	    lower_incl = mid
	} else {
	    return Err(SeatError::InvalidInput(
		format!("invalid direction character '{}'", choice)));
	}
    }
    if upper_excl == lower_incl + 1 {
	Ok(lower_incl)
    } else {
	Err(SeatError::InvalidInput(
	    "insufficient seat directions".to_string()))
    }
}

fn decode_seat(directions: &str) -> Result<i32, SeatError> {
    let r = binary_search(0, 128, 'F', 'B', &directions[0..7])?;
    let c = binary_search(0, 8, 'L', 'R', &directions[7..])?;
    Ok(r * 8 + c)
}

fn part1(seats: &BTreeSet<i32>) -> Result<(), SeatError> {
    match seats.iter().next_back() {
	Some(n) => {
	    println!("Part 1: largest seat ID is {}", n);
	    Ok(())
	}
	None => Err(SeatError::NoInput),
    }
}

fn part2(seats: &BTreeSet<i32>) -> Result<(), SeatError> {
    for seat in seats.iter() {
	let following = seat + 1;
	if !seats.contains(&following) {
	    println!("Part 2: my seat is {}", following);
	    return Ok(());
	}
    }
    Err(SeatError::InvalidInput(
	"there are no gaps in the boarding passes".to_string()))
}

fn run() -> Result<(), SeatError> {
    let seats = io::BufReader::new(io::stdin()).lines()
	.map(|x| match x {
	    Err(source) => Err(SeatError::ReadError{source}),
	    Ok(line) => decode_seat(&line.as_str()),
	})
	.collect::<Result<BTreeSet<i32>, _>>()?;
    part1(&seats)?;
    part2(&seats)?;
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
