use std::collections::BTreeMap;
use std::io::BufRead;
use std::io;

fn differences(ratings: &Vec<i64>) -> Vec<(i64, i64)> {
    let mut result: Vec<(i64, i64)> = Vec::new();
    result.reserve(ratings.len());
    let mut last = 0;
    for rating in ratings {
	result.push((rating-last, *rating));
	last = *rating
    }
    result.push((3, last+3));
    result
}


fn read_i64(thing: Result<String, std::io::Error>) -> Result<i64, String> {
    match thing {
	Err(e) => Err(format!("I/O error: {}", e)),
	Ok(line) => match line.parse::<i64>() {
	    Err(e) => Err(format!("unable to parse '{}' as an integer: {}", line, e)),
	    Ok(n) => Ok(n),
	}
    }
}

fn sorted_integer_input() -> Result<Vec<i64>, String> {
    let mut items: Vec<i64> = match io::BufReader::new(io::stdin())
	.lines().map(read_i64).collect() {
	    Err(e) => return Err(e),
	    Ok(numbers) => numbers,
	};
    items.sort();
    Ok(items)
}

fn part1(ratings: &Vec<i64>) -> (Vec<(i64, i64)>, i64) {
    let diffs = differences(ratings);
    let my_device_rating: i64 = (*diffs.last().unwrap()).1;
    println!("Part 1: my device rating is {}", my_device_rating);
    let mut counts: BTreeMap<i64, usize> = BTreeMap::new();
    for (d, _) in &diffs {
	match d {
	    1 | 2 | 3 => {
		*counts.entry(*d).or_insert(0) += 1;
	    }
	    _ => {
		panic!(format!("unexpected diff {}", d));
	    }
	}
    }
    let solution: usize = counts.get(&1).unwrap_or(&0) * counts.get(&3).unwrap_or(&0);
    println!("Part 1: answer is {}", solution);
    (diffs, my_device_rating)
}

fn add_rating(r: i64, prev: &mut Option<i64>,
	      runs: &mut Vec<i64>,
	      run_length: &mut i64) {
    if let Some(p) = *prev {
	if p + 1 == r {
	    (*run_length) += 1;
	} else {
	    runs.push(*run_length);
	    *run_length = 0;
	}
    }
    *prev = Some(r);
}

fn find_run_lengths(ratings: &Vec<i64>) -> Vec<i64> {
    let mut runs: Vec<i64> = Vec::new();
    let mut prev: Option<i64> = None;
    let mut run_length: i64 = 0;
    for r in ratings {
	add_rating(*r, &mut prev, &mut runs, &mut run_length);
    }
    add_rating(-1, &mut prev, &mut runs, &mut run_length);
    runs
}


fn bookend(ratings: &Vec<i64>, first: i64, last: i64) -> Vec<i64> {
    itertools::chain(itertools::chain(std::iter::once(first),
				      ratings.iter().cloned()),
		     std::iter::once(last))
	.collect()
}

struct TribEval {
    known: BTreeMap<i64, i64>
}

impl TribEval {
    fn new() -> TribEval {
	let mut m = BTreeMap::new();
	m.insert(0, 1);
	m.insert(1, 1);
	m.insert(2, 2);
	TribEval {
	    known: m,
	}
    }

    fn tribonacci(&mut self, wanted: i64) -> i64 {
	let mut largest = match self.known.keys().next_back() {
	    None => {
		panic!("trib_values is uninitialized");
	    }
	    Some(curr) => *curr,
	};
	while largest < wanted {
	    largest += 1;
	    let v = self.known.values().rev().take(3).sum();
	    match self.known.insert(largest, v) {
		Some(oldval) => {
		    panic!("conflicting updates for tribonacci values");
		}
		None => ()		// this is the usual case.
	    };
	}
	match self.known.get(&wanted) {
	    Some(y) => *y,
	    None => {
		panic!(format!("tribonacci value for {} was not populated", wanted));
	    }
	}
    }
}


fn self_test() {
    let mut te = TribEval::new();
    assert_eq!(te.tribonacci(0), 1);
    assert_eq!(te.tribonacci(1), 1);
    assert_eq!(te.tribonacci(2), 2);
    assert_eq!(te.tribonacci(3), 4);
    assert_eq!(te.tribonacci(4), 7);
    assert_eq!(te.tribonacci(5), 13);
    assert_eq!(te.tribonacci(6), 24);
}



fn part2(ratings: &Vec<i64>, my_device_rating: i64) -> i64 {
    // Based on a hint from reddit.com/r/AdventOfCode.
    let runs = find_run_lengths(&bookend(ratings, 0, my_device_rating));
    let mut result: i64 = 1;
    let mut te = TribEval::new();
    for run_len in runs {
	result = result * te.tribonacci(run_len);
    }
    println!("Part 2: answer is {}", result);
    result
}


fn run() -> Result<(), String> {
    self_test();
    let ratings = sorted_integer_input()?;
    let (diffs, my_device_rating) = part1(&ratings);
    part2(&ratings, my_device_rating);
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
