extern crate itertools;
extern crate lazy_static;
extern crate regex;

use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::io;
use std::io::BufRead;
use std::ops::RangeInclusive;

lazy_static! {
    static ref FIELD_RE: Regex = Regex::new(r"^([^:]*): (\d+)-(\d+) or (\d+)-(\d+)$").unwrap();
}

#[derive(Clone)]
struct Field {
    name: String,
    first: RangeInclusive<i32>,
    second: RangeInclusive<i32>,
}

impl Field {
    fn is_valid_value(&self, v: &i32) -> bool {
        self.first.contains(v) || self.second.contains(v)
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {}-{} or {}-{}",
            self.name,
            self.first.start(),
            self.first.end(),
            self.second.start(),
            self.second.end()
        )
    }
}

impl Field {
    fn new(s: &str) -> Result<Field, String> {
        match FIELD_RE.captures(s) {
            None => Err(format!("'{}' is not a valid field input line", s)),
            Some(caps) => Ok(Field {
                name: caps[1].to_string(),
                first: RangeInclusive::new(
                    caps[2].parse().expect("invalid from1"),
                    caps[3].parse().expect("invalid to1"),
                ),
                second: RangeInclusive::new(
                    caps[4].parse().expect("invalid from2"),
                    caps[5].parse().expect("invalid to2"),
                ),
            }),
        }
    }
}

#[derive(Clone)]
struct Ticket {
    values: Vec<i32>,
}

impl fmt::Display for Ticket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&itertools::join(&self.values, ","))
    }
}

fn read_ticket(s: &str) -> Result<Ticket, String> {
    let mut vs: Vec<i32> = Vec::new();
    for value in s.split(',') {
        match value.parse() {
            Err(e) => {
                return Err(format!("failed to parse value '{}': {}", value, e));
            }
            Ok(n) => {
                vs.push(n);
            }
        }
    }
    Ok(Ticket { values: vs })
}

struct Input {
    fields: Vec<Field>,
    my_ticket: Ticket,
    nearby: Vec<Ticket>,
}

impl Input {
    fn is_valid_value(&self, v: &i32) -> bool {
        self.fields.iter().any(|f| f.is_valid_value(v))
    }
}

impl fmt::Display for Input {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}\n\nyour ticket:\n{}\n\nnearby tickets:\n{}",
            itertools::join(self.fields.iter(), "\n"),
            self.my_ticket,
            itertools::join(self.nearby.iter(), "\n")
        )
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
            Some(s) if s.is_empty() => {
                println!("reached the end of the field descriptions");
                break;
            }
            Some(s) => {
                println!("parsing a field description '{}'", s);
                match Field::new(s) {
                    Err(e) => return Err(e),
                    Ok(f) => {
                        fields.push(f);
                    }
                }
            }
        }
    }

    match lines.next() {
        Some(s) if s == "your ticket:" => (),
        Some(s) => {
            return Err(format!("unexpected: {}", s));
        }
        None => {
            return premature_eof;
        }
    };
    let my_ticket = match lines.next() {
        None => {
            return premature_eof;
        }
        Some(s) => match read_ticket(s) {
            Err(e) => {
                return Err(e);
            }
            Ok(t) => t,
        },
    };
    match lines.next() {
        Some(s) if s.is_empty() => (),
        Some(s) => {
            return Err(format!("expected a blank line, not {}", s));
        }
        None => {
            return premature_eof;
        }
    };
    match lines.next() {
        Some(s) if s == "nearby tickets:" => (),
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
                Err(e) => {
                    return Err(e);
                }
                Ok(t) => {
                    nearby.push(t);
                }
            },
        }
    }

    Ok(Input {
        fields,
        my_ticket,
        nearby,
    })
}

fn part1(input: &Input) -> Result<Vec<Ticket>, String> {
    let mut invalid_values: Vec<i32> = Vec::new();
    let mut valid_tickets: Vec<Ticket> = Vec::new();
    for t in &input.nearby {
        let mut this_ticket_valid = true;
        for v in &t.values {
            if !input.is_valid_value(v) {
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

fn compute_candidates(
    fields_by_name: &HashMap<String, Field>,
    field_position: &HashMap<String, i32>,
    nf: usize,
    valid_tickets: &Vec<Ticket>,
) -> HashMap<String, Vec<i32>> {
    let mut candidates: HashMap<String, Vec<i32>> = HashMap::new();
    let known_field_numbers: HashSet<i32> = field_position.values().cloned().collect();
    for i in 0..nf {
        if known_field_numbers.contains(&(i as i32)) {
            continue;
        }
        for (field_name, f) in fields_by_name.iter() {
            if field_position.contains_key(field_name) {
                continue;
            }
            let mut invalid: bool = false;
            for t in valid_tickets {
                let v = &t.values.get(i).unwrap();
                if !f.is_valid_value(v) {
                    invalid = true;
                    break;
                }
            }
            if !invalid {
                candidates
                    .entry(field_name.to_string())
                    .or_insert(Vec::new())
                    .push(i as i32)
            }
        }
    }
    candidates
}

fn part2(input: &Input, valid_tickets: &Vec<Ticket>) -> Result<(), String> {
    let nf: usize = valid_tickets.get(0).unwrap().values.len();
    let odd_length_tickets: usize = valid_tickets
        .iter()
        .filter(|t| t.values.len() != nf)
        .count();
    assert_eq!(0, odd_length_tickets);
    let mut fields_by_name: HashMap<String, Field> = HashMap::new();
    let mut fields_todo: HashSet<String> = HashSet::new();
    let mut field_positions: HashMap<String, i32> = HashMap::new();
    for f in &input.fields {
        fields_by_name.insert(f.name.clone(), f.clone());
        fields_todo.insert(f.name.clone());
    }
    for iter in 1.. {
        if fields_todo.is_empty() {
            break;
        }
        let mut progress = false;
        println!(
            "iteration {}: unknown: {} fields ({:?})\nknown: {:?}",
            iter,
            fields_todo.len(),
            fields_todo,
            field_positions
        );

        let candidates = compute_candidates(&fields_by_name, &field_positions, nf, valid_tickets);
        for (name, cands) in candidates {
            println!("candidates for field {}: {:?}", name, cands);
            if cands.len() == 1 {
                let only = cands.first().unwrap();
                field_positions.insert(name.to_string(), *only);
                println!(
                    "field {} must be at position {:?}",
                    name,
                    &field_positions.get(&name).unwrap()
                );
                assert!(fields_todo.remove(&name));
                progress = true;
            }
        }
        assert!(progress);
    }
    let ans: i64 = field_positions
        .iter()
        .filter(|(name, _)| name.starts_with("departure"))
        .map(|(_, pos)| input.my_ticket.values[*pos as usize] as i64)
        .product();
    println!("Part 2: product = {}", ans);
    Ok(())
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
    }
    let input = read_input(input_lines)?;
    println!("Day 16: input:\n{}", input);
    let valid_tickets = part1(&input)?;
    part2(&input, &valid_tickets)?;
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
