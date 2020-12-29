use std::io;
use std::io::BufRead;
use std::collections::BTreeSet;

fn partition(mut lower_incl: i32, mut upper_excl: i32,
	     lower_directive: char, upper_directive: char,
	     choices: &str) -> (i32, i32) {
    for choice in choices.chars() {
	if upper_excl == lower_incl + 1 {
	    panic!("we already have the answet but there are more directives remaining");
	}
	let mid = (upper_excl + lower_incl) / 2;
	if choice == lower_directive {
	    upper_excl = mid
	} else if choice == upper_directive {
	    lower_incl = mid
	} else {
	    panic!("unexpected choice '{}' in '{}'",
		   choice, choices);
	}
    }
    return (lower_incl, upper_excl)
}

fn single_value(partition_result: (i32, i32)) -> i32 {
    match partition_result {
	(lower_incl, upper_excl) => {
	    if upper_excl == lower_incl + 1 {
		return lower_incl
	    }
	    panic!("single_value: gap is not 1");
	}
    }
}

fn decode_seat(directions: &str) -> (i32, i32) {
    return (single_value(partition(0, 128, 'F', 'B', &directions[0..7])),
	    single_value(partition(0, 8, 'L', 'R', &directions[7..])))
}

fn seat_id(row: i32, col: i32) -> i32 {
    row * 8 + col
}

fn part1(seats: &BTreeSet<i32>) {
    match seats.iter().next_back() {
	Some(n) => {
	    println!("Part 1: largest seat ID is {}", n);
	}
	None => {
	    panic!("Part 1: there are no boarding passes!");
	}
    }
}

fn part2(seats: &BTreeSet<i32>) {
    for seat in seats.iter() {
	let following = seat + 1;
	if !seats.contains(&following) {
	    println!("Part 2: my seat is {}", following);
	    return;
	}
    }
    panic!("Part 2: there are no gaps in the boarding passes");
}

fn run() -> Result<(), std::io::Error> {
    let reader = io::BufReader::new(io::stdin());
    let seats: BTreeSet<i32> = reader.lines()
	.map(|l| l.unwrap())
	.map(|l| decode_seat(&l.as_str()))
	.map(|(row, col)| seat_id(row, col))
	.collect();
    part1(&seats);
    part2(&seats);
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
