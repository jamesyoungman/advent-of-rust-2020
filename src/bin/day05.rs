use std::io;
use std::io::BufRead;
use std::collections::BTreeSet;

fn binary_search(mut lower_incl: i32, mut upper_excl: i32,
	     lower_directive: char, upper_directive: char,
	     choices: &str) -> Result<i32, &'static str> {
    if lower_incl >= upper_excl {
	return Err("empty starting range");
    }
    for choice in choices.chars() {
	if upper_excl == lower_incl + 1 {
	    return Err("overdetermined: too many directives");
	}
	let mid = lower_incl + (upper_excl - lower_incl)/2;
	if choice == lower_directive {
	    upper_excl = mid
	} else if choice == upper_directive {
	    lower_incl = mid
	} else {
	    return Err("invalid choice");
	}
    }
    if upper_excl == lower_incl + 1 {
	return Ok(lower_incl)
    }
    return Err("undetermined, insuffiicient directives");
}

fn seat_id(row: i32, col: i32) -> i32 {
    row * 8 + col
}

fn decode_seat(directions: &str) -> Result<i32, &'static str> {
    match (binary_search(0, 128, 'F', 'B', &directions[0..7]),
	   binary_search(0, 8, 'L', 'R', &directions[7..])) {
	(Ok(r), Ok(c)) => Ok(seat_id(r, c)),
	(Err(e), _) => Err(e),
	(_, Err(e)) => Err(e),
    }
}

fn part1(seats: &BTreeSet<i32>) -> Result<(), &'static str> {
    match seats.iter().next_back() {
	Some(n) => {
	    println!("Part 1: largest seat ID is {}", n);
	    Ok(())
	}
	None => Err("Part 1: there are no boarding passes!"),
    }
}

fn part2(seats: &BTreeSet<i32>) -> Result<(), &'static str> {
    for seat in seats.iter() {
	let following = seat + 1;
	if !seats.contains(&following) {
	    println!("Part 2: my seat is {}", following);
	    return Ok(());
	}
    }
    Err("Part 2: there are no gaps in the boarding passes")
}

fn run() -> Result<(), String> {
    let seats_or_error: Result<BTreeSet<i32>, String> = io::BufReader::new(io::stdin()).lines()
	.map(|x| match x {
	    Err(e) => Err(format!("I/O error: {}", e)),
	    Ok(line) => decode_seat(&line.as_str()).map_err(str::to_string),
	})
	.collect();
    match seats_or_error {
	Ok(seats) => {
            part1(&seats).map_err(str::to_string)?;
            part2(&seats).map_err(str::to_string)?;
            return Ok(());
	}
	Err(e) => Err(e),
    }
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
