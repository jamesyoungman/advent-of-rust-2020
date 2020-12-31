extern crate itertools;

use std::io;
use std::collections::VecDeque;
use std::io::BufRead;


fn read_i64(thing: Result<String, std::io::Error>) -> Result<i64, String> {
    match thing {
	Err(e) => Err(format!("I/O error: {}", e)),
	Ok(line) => match line.parse::<i64>() {
	    Err(e) => Err(format!("unable to parse '{}' as an integer: {}", line, e)),
	    Ok(n) => Ok(n),
	}
    }
}

fn find_pair(candidates: &VecDeque<i64>, n: &i64) -> Option<(i64, i64)> {
    for v in candidates.iter() {
	let diff: i64 = n - v;
	if (diff != *v) && candidates.contains(&diff) {
	    return Some((*v, diff));
	}
    }
    return None
}

fn solve1(all_input: &Vec<i64>, preamble_len: usize) -> Option<i64> {
    let mut candidates: VecDeque<i64> = VecDeque::new();
    for n in all_input {
	if candidates.len() > preamble_len {
	    candidates.pop_back();
	    print!("{} -> ", n);
	    match find_pair(&candidates, &n) {
		None => {
		    println!("no match");
		    return Some(*n);
		}
		Some((x, y)) => {
		    println!("{},{}", x, y);
		}
	    }
	}
	candidates.push_front(*n);
    }
    None
}

fn part1(all_input: &Vec<i64>, preamble_len: usize) {
    match solve1(all_input, preamble_len) {
	Some(n) => {
	    println!("Part 1: invalid number is {}", n);
	}
	None => {
	    println!("Part 1: did not find the invalid number");
	}
    }
}

fn run() -> Result<(), String> {
    let numbers: Vec<i64> = match io::BufReader::new(io::stdin())
	.lines().map(read_i64).collect() {
	    Err(e) => return Err(e),
	    Ok(numbers) => numbers,
	};
    part1(&numbers, 25);
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
