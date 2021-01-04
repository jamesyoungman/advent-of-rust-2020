extern crate regex;

use std::collections::HashMap;
use std::collections::VecDeque;
use std::io;
use std::io::BufRead;

fn playgame(start_numbers: &[usize], turns: &usize, verbose: bool) -> Option<usize> {
    let mut turns_spoken: HashMap<usize, VecDeque<usize>> = HashMap::new();
    let mut last_number: Option<usize> = None;
    let mut this_number: usize;
    if verbose {
	println!("\nPlaying {} turns from {:?}", turns, start_numbers);
    }
    for turn in 0..*turns {
	if verbose {
	    print!("Turn {}: ", turn+1);
	}
	if turn < start_numbers.len() {
	    this_number = start_numbers[turn];
	    if verbose {
		print!("saying start number {}; ", this_number);
	    }
	} else {
	    let when = turns_spoken.entry(last_number.unwrap()).or_insert(VecDeque::new());
	    if when.len() == 1 {
		if verbose {
		    print!("number {} has never been said before, saying zero; ",
			   last_number.unwrap());
		}
		this_number = 0;
	    } else {
		let mut items = when.iter().rev();
		let later = items.next().unwrap();
		let earlier = items.next().unwrap();
		this_number = later - earlier;
		if verbose {
		    print!("number {} was previously said in turn {}, hence saying {}; ",
			   last_number.unwrap(), earlier, this_number);
		}
	    }
	}
	last_number = Some(this_number);
	let history = &mut turns_spoken.entry(this_number).or_insert(VecDeque::new());
	if verbose {
	    print!("history of {} is {:?}; ", this_number, history);
	}
	history.push_back(turn);
	while history.len() > 2 {
	    history.pop_front();
	}
	if verbose {
	    println!("last_number is now {:?}.", last_number);
	}
    }
    last_number
}

fn run_one_test(label: &str,
		start_numbers: &[usize], turns: &usize, expected: &Option<usize>,
		verbose: bool) -> Result<(), String> {
    let got = playgame(start_numbers, turns, verbose);
    if got != *expected {
	Err(format!("FAIL: '{}': run_one_test: {:?} turn {}: expected {:?}, got {:?}",
		    label, start_numbers, turns, expected, got))
    } else {
	Ok(())
    }
}

fn runtests(verbose: bool) -> Result<(), String> {
    let scenarios: &[(&str, &[usize], usize, Option<usize>)] = &[
	("a", &[0, 3, 6], 0, None),
	("b", &[0, 3, 6], 1, Some(0)),
	("c", &[0, 3, 6], 2, Some(3)),
	("d", &[0, 3, 6], 3, Some(6)),
	("e", &[0, 3, 6], 4, Some(0)),
	("f", &[0, 3, 6], 5, Some(3)),
	("g", &[0, 3, 6], 6, Some(3)),
	("h1", &[0, 3, 6], 7, Some(1)),
	("h2", &[0, 3, 6], 8, Some(0)),
	("h3", &[0, 3, 6], 9, Some(4)),
	("h4", &[0, 3, 6], 10, Some(0)),
	("i", &[0, 3, 6], 2020, Some(436)),
	("j", &[1,3,2], 2020, Some(1)),
	("j", &[2,1,3], 2020, Some(10)),
	("k", &[1,2,3], 2020, Some(27)),
	("l", &[2,3,1], 2020, Some(78)),
	("m", &[3,2,1], 2020, Some(438)),
	("n", &[3,1,2], 2020, Some(1836)),
    ];
    for scenario in scenarios {
	let (label, start_numbers, turns, expected): &(&str, &[usize], usize, Option<usize>)
	    = scenario;
	match run_one_test(label, start_numbers, turns, expected, verbose) {
	    Ok(()) => (),
	    Err(_) => {
		return run_one_test(label, start_numbers, turns, expected, true);
	    }
	}
    }
    Ok(())
}

fn self_test() -> Result<(), String> {
    runtests(true)
}


fn run() -> Result<(), String> {
    self_test()?;
    let input = io::BufReader::new(io::stdin());
    let start_numbers: Vec<usize> = match input.lines().next() {
	None => return Err("no numbers were specified in the input".to_string()),
	Some(Err(e)) => return Err(format!("I/O error: {}", e)),
	Some(Ok(s)) => s.split(",")
	    .map(|s| (*s).parse::<usize>().expect("failed to parse integer"))
	    .collect(),
    };
    for (part, which_num) in vec![(1, 2020), (2, 30000000)] {
	println!("Part {}: number = {:?}",
		 part, playgame(&start_numbers, &which_num, false));
    }
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
