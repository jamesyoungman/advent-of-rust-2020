extern crate itertools;
extern crate lazy_static;
extern crate regex;

use itertools::Itertools;
use lazy_static::lazy_static;
use std::fmt;
use regex::Regex;
use std::ops::RangeInclusive;
use std::io;
use std::io::BufRead;

lazy_static! {
    static ref FIELD_RE: Regex = Regex::new(
	r"^([^:]*): (\d+)-(\d+) or (\d+)-(\d+)$").unwrap();
}

struct Field {
    name: String,
    first: RangeInclusive<i32>,
    second: RangeInclusive<i32>,
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
	write!(f, "{}: {}-{} or {}-{}",
	       self.name,
	       self.first.start(), self.first.end(),
	       self.second.start(), self.second.end())
    }
}


impl Field {
    fn new(s: &str) -> Result<Field, String> {
	match FIELD_RE.captures(s) {
	    None => Err(format!("'{}' is not a valid field input line", s)),
	    Some(caps) => {
		Ok(Field {
		    name: caps[1].to_string(),
		    first: RangeInclusive::new(caps[2].parse().expect("invalid from1"),
					       caps[3].parse().expect("invalid to1")),
		    second: RangeInclusive::new(caps[4].parse().expect("invalid from2"),
						caps[5].parse().expect("invalid to2")),
		})
	    }
	}
    }
}

#[derive(Clone)]
struct Ticket {
    values: Vec<i32>
}

impl fmt::Display for Ticket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
	f.write_str(&itertools::join(&self.values, ","))
    }
}

fn read_ticket(s: &str) -> Result<Ticket, String> {
    let mut vs: Vec<i32> = Vec::new();
    for value in s.split(",") {
	match value.parse() {
	    Err(e) => {
		return Err(format!("failed to parse value '{}': {}",
				   value, e));
	    }
	    Ok(n) => {
		vs.push(n);
	    }
	}
    }
    Ok(Ticket{
	values: vs
    })
}

struct Input {
    fields: Vec<Field>,
    my_ticket: Ticket,
    nearby: Vec<Ticket>,
}

impl Input {
    fn is_valid_value(&self, v: &i32) -> bool {
	for f in &self.fields {
	    if f.first.contains(v) || f.second.contains(v) {
		return true;
	    }
	}
	return false;
    }
}


impl fmt::Display for Input {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
	write!(f, "{}\n\nyour ticket:\n{}\n\nnearby tickets:\n{}",
	       itertools::join(self.fields.iter(), "\n"),
	       self.my_ticket,
	       itertools::join(self.nearby.iter(), "\n"))
	}
}

fn read_input(input: Vec<String>) -> Result<Input, String> {
    // Read the fields.
    let mut fields: Vec<Field> = Vec::new();
    let mut lines = input.iter();
    let premature_eof = Err("premature end-of-input".to_string());

    loop {
	match lines.next() {
    	    None => {
    		return Err("my ticket is not here".to_string());
    	    }
    	    Some(s) if s == "" => {
		println!("reached the end of the field descriptions");
		break;
	    }
    	    Some (s) => {
		println!("parsing a field description '{}'", s);
    		match Field::new(s) {
    		    Err(e) => {
    			return Err(e)
    		    }
    		    Ok(f) => {
    			fields.push(f);
    		    }
    		}
    	    }
	}
    }

    match lines.next() {
    	Some (s) if s == "your ticket:" => (),
	Some(s) => {
	    return Err(format!("unexpected: {}", s));
	}
	None => {
	    return premature_eof;
	}
    };
    let my_ticket = match lines.next() {
	None => { return premature_eof; }
	Some(s) => match read_ticket(s) {
	    Err(e) => { return Err(e); }
	    Ok(t) => t,
	}
    };
    match lines.next() {
	Some(s) if s == "" => (),
	Some(s) => { return Err(format!("expected a blank line, not {}", s)); }
	None => { return premature_eof; }
    };
    match lines.next() {
    	Some (s) if s == "nearby tickets:" => (),
	Some(s) => {
	    return Err(format!("unexpected: {}", s));
	}
	None => {
	    return premature_eof;
	}
    };

    let mut nearby: Vec<Ticket> = Vec::new();
    loop {
	match lines.next() {
	    None => {
		break;
	    }
	    Some(s) => match read_ticket(s) {
		Err(e) => { return Err(e); }
		Ok(t) => {
		    nearby.push(t);
		}
	    }
	}
    }

    Ok(Input{
	fields,
	my_ticket,
	nearby
    })
}

fn part1(input: &Input) -> Result<Vec<Ticket>, String> {
    let mut invalid_values: Vec<i32> = Vec::new();
    let mut valid_tickets: Vec<Ticket> = Vec::new();
    for t in &input.nearby {
	let mut this_ticket_valid = true;
	for v in &t.values {
	    if !input.is_valid_value(&v) {
		invalid_values.push(*v);
		this_ticket_valid = false;
	    }
	}
	if this_ticket_valid {
	    valid_tickets.push(t.clone());
	}
    }
    let total: i32 = invalid_values.iter().sum();
    println!("Part 1: total {}", total);
    Ok(valid_tickets)
}


fn run() -> Result<(), String> {
    let mut input_lines: Vec<String> = Vec::new();
    for input_item in io::BufReader::new(io::stdin()).lines() {
	match input_item {
	    Err(e) => {
		return Err(format!("I/O error: {}", e));
	    }
	    Ok(item) => {
		input_lines.push(item);
	    }
	}
    };
    let input = read_input(input_lines)?;
    println!("Day 16: input:\n{}", input);
    let valid_tickets = part1(&input)?;
    //22part2()?;
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
