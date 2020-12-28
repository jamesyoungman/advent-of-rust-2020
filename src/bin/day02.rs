use std::io;
extern crate regex;
use std::io::BufRead;

use regex::Regex;


fn atpos(pos: usize, expected: &str, passwd: &str) -> usize {
    match passwd.get(pos-1..pos) {
	Some(got) => {
	    (expected == got) as usize
	}
	None => 0,
    }
}


fn valid2(n1: usize, n2: usize, ch: &str, passwd: &str) -> bool {
    return (atpos(n1, ch, passwd) + atpos(n2, ch, passwd)) == 1;
}

fn valid1(n1: usize, n2: usize, ch: &str, passwd: &str) -> bool {
    let actual = passwd.matches(ch).count();
    return actual >= n1 && actual <= n2;
}

fn run() -> Result<(), std::io::Error> {
    // Example input lines:
    // 1-3 a: abcde
    // 1-3 b: cdefg
    // 2-9 c: ccccccccc
    let mut total: u32 = 0;
    let mut count1: u32 = 0;
    let mut count2: u32 = 0;
    let re = Regex::new(r"^(\d+)-(\d+) (.): (.*)$").unwrap();
    let reader = io::BufReader::new(io::stdin());
    for line_or_fail in reader.lines() {
	match line_or_fail {
	    Ok(line) => {
		let s = line.trim_end();
		total += 1;
		match re.captures(&s) {
		    None => {
			panic!("error: invalid input line {}", s);
		    }
		    Some(cap) => {
			let n1: usize = cap[1].parse().unwrap();
			let n2: usize = cap[2].parse().unwrap();
			if n1 < 1 || n2 < 1 {
			    panic!("{} is not a valid rule", s);
			}
			let ch = &cap[3];
			let password = &cap[4];
			if valid1(n1, n2, ch, password) {
			    count1 += 1;
			}
			if valid2(n1, n2, ch, password) {
			    count2 += 1;
			}
		    }
		}
	    }
	    Err(e) => {
		return Err(e);
	    }
	}
    }
    println!("Part 1: {} valid passwords out of {}", count1, total);
    println!("Part 2: {} valid passwords out of {}", count2, total);
    return Ok(());
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
