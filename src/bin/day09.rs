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

fn solve1<T>(input: T, preamble_len: usize)
	     -> Option<i64> where T: IntoIterator<Item=i64> {
    let mut candidates: VecDeque<i64> = VecDeque::new();
    for n in input {
	if candidates.len() > preamble_len {
	    candidates.pop_back();
	    if find_pair(&candidates, &n).is_none() {
		return Some(n);
	    }
	}
	candidates.push_front(n);
    }
    None
}

fn min_max_sum<U: Ord + std::ops::AddAssign + Copy,
	       T: std::iter::Iterator<Item=U>>(mut input: T) -> Option<(U, U, U)> {
    match input.next() {
	Some(n) => {
	    let mut acc: (U, U, U) = (n, n, n);
	    while let Some(n) = input.next() {
		acc.2 += n;
		acc = (std::cmp::min(acc.0, n),
		       std::cmp::max(acc.1, n),
		       acc.2);
	    }
	    Some(acc)
	}
	None => None
    }
}


fn solve2(all_input: &Vec<i64>, target: i64) -> Option<(i64, i64)> {
    for windowsize in 2..all_input.len() {
	for w in all_input.windows(windowsize) {
	    if let Some((wmin, wmax, wsum)) = min_max_sum(w.iter().cloned()) {
		if wsum == target {
		    return Some((wmin, wmax));
		}
	    }
	}
    }
    None
}


fn run() -> Result<(), String> {
    let numbers: Vec<i64> = match io::BufReader::new(io::stdin())
	.lines().map(read_i64).collect() {
	    Err(e) => return Err(e),
	    Ok(numbers) => numbers,
	};
    let preamble_len = 25;
    let n = match solve1(numbers.iter().cloned(), preamble_len) {
	Some(n) => n,
	None => {
	    return Err("Part 1: did not find the invalid number".to_string());
	}
    };
    println!("Part 1: invalid number is {}", n);
    match solve2(&numbers, n) {
	Some((least, most)) => {
	    println!("Part 2: {} + {} = {}", least, most, (least+most));
	    Ok(())
	},
	None => {
	    Err("did not find a solution to part 2".to_string())
	}
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
