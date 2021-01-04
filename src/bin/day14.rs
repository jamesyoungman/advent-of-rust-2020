extern crate lazy_static;
extern crate regex;

use lazy_static::lazy_static;
use regex::Regex;
use std::collections::BTreeMap;
use std::fmt;
use std::io;
use std::io::BufRead;

lazy_static! {
    static ref SETMASK_RE: Regex = Regex::new(r"mask = ([01X]{36})$").unwrap();
    // Sigh, a character class in Rust cannot just contain ], it must be escaped.
    static ref STORE_RE: Regex = Regex::new(
	r"mem[\[]([0-9]+)[\]] = ([0-9]+)$").unwrap();
}

enum Operation {
    SetMask(String),
    Store(i64, i64),
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
	match self {
	    Operation::SetMask(s) => write!(f, "mask = {}", s),
	    Operation::Store(addr, val) => write!(f, "mem[{}] = {}", addr, val),
	}
    }
}


fn parse_line(s: &str) -> Result<Operation, String> {
    match SETMASK_RE.captures(s) {
	Some(caps) => Ok(Operation::SetMask(caps[1].to_string())),
	None => match STORE_RE.captures(s) {
	    Some(caps) => Ok(Operation::Store(
		caps[1].parse().expect("invalid address"),
		caps[2].parse().expect("invalid value"))),
	    None => Err(format!("unrecognised line: '{}'", s))
	}
    }
}

fn execute_v2<'a, OPZ>(operations: OPZ) -> Result<i64, String>
where OPZ: Iterator<Item = &'a Operation> {
    let mut data: BTreeMap<i64, i64> = BTreeMap::new();
    let mut or_mask: i64 = 0;
    let mut float_mask: i64 = 0;
    for op in operations {
	match op {
	    Operation::SetMask(s) => {
		or_mask = 0;
		float_mask = 0;
		for (bitpos, ch) in s.chars().rev().enumerate() {
		    let bit = 1 << bitpos;
		    match ch {
			'X' => float_mask |= bit,
			'1' => or_mask |= bit,
			'0' => (),
			_ => {
			    panic!(format!("v2: unexpected character '{}'", ch));
			}
		    }
		}
	    }
	    Operation::Store(addr, val) => {
		let mut locations : Vec<i64> = vec![addr | or_mask];
		for bitnum in 0..=36 {
		    let mask = 1 << bitnum;
		    if (float_mask & mask) != 0 {
			let mut updated_locations: Vec<i64> = Vec::new();
			for loc in locations {
			    updated_locations.push(loc & !mask); // X turns to 0
			    updated_locations.push(loc | mask); // X turns to 1
			}
			locations = updated_locations;
		    }
		}
		for loc in locations {
		    data.insert(loc, *val);
		}

	    }
	}
    }
    Ok(data.values().sum())
}

fn execute_v1<'a, OPZ>(operations: OPZ) -> Result<i64, String>
where OPZ: Iterator<Item = &'a Operation> {
    let mut data: BTreeMap<i64, i64> = BTreeMap::new();
    let mut or_mask: i64 = 0;
    let mut and_mask: i64 = 0;
    for op in operations {
	match op {
	    Operation::SetMask(s) => {
		and_mask = 0;
		or_mask = 0;
		for (bitpos, ch) in s.chars().rev().enumerate() {
		    let bit = 1 << bitpos;
		    match ch {
			'X' => { and_mask |= bit; }
			'1' => { or_mask |= bit; and_mask |= bit; }
			'0' => (),
			_ => { panic!(format!("v1: unexpected character '{}'", ch)); }
		    }
		}

	    }
	    Operation::Store(addr, val) => {
		let v = (val & and_mask) | or_mask;
		data.insert(*addr, v);
	    }
	}
    }
    Ok(data.values().sum())
}

fn read_input(reader: impl BufRead) -> Result<Vec<Operation>, String> {
    let mut ops: Vec<Operation> = Vec::new();
    for thing in reader.lines() {
	match thing {
	    Err(e) => return Err(format!("I/O error: {}", e)),
	    Ok(line) => match parse_line(&line) {
		Ok(op) => ops.push(op),
		Err(e) => return Err(e),
	    }
	}
    }
    Ok(ops)
}

fn run() -> Result<(), String> {
    let operations = read_input(io::BufReader::new(io::stdin()))
	.expect("unable to read operations list");
    println!("Part 1: sum = {}",
	     execute_v1(operations.iter()).expect("part 1 execute failed"));
    println!("Part 2: sum = {}",
	     execute_v2(operations.iter()).expect("part 2 execute failed"));
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
