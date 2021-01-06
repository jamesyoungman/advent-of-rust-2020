extern crate lazy_static;
extern crate regex;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::io::BufRead;
use std::io;
use std::fmt;

lazy_static! {
    static ref RULE_RE: Regex = Regex::new(r"^(\d+): (.*)$").expect("RULE_RE");
    static ref LIT_RE: Regex = Regex::new("\"(.)\"$").expect("LIT_RE");
    static ref ALT_RE: Regex = Regex::new(r"([^|]+)[|]([^|]+)$").expect("ALT_RE");
}

type RuleId = i32;

#[derive(PartialEq, Debug)]
enum Rule {
    Sequence(Vec<RuleId>),
    Alternative(Vec<RuleId>, Vec<RuleId>),
    Literal(char)
}

fn seq_items(s: &str) -> Result<Vec<RuleId>, String> {
    let mut result = Vec::new();
    for field in s.split(" ") {
	match field.parse() {
	    Ok(n) => {
		result.push(n);
	    }
	    Err(e) => {
		return Err(format!("failed to parse integer '{}': {}",
				   field, e));
	    }
	}
    }
    Ok(result)
}

fn seq_fmt(seq: &Vec<RuleId>) -> String {
    itertools::join(seq.iter(), " ")
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
	match self {
	    Rule::Sequence(idlist) => f.write_str(&seq_fmt(&idlist)),
	    Rule::Alternative(left, right) => write!(f, "{} | {}",
						     seq_fmt(left), seq_fmt(right)),
	    Rule::Literal(ch) => write!(f, "\"{}\"", ch),
	}
    }
}

fn parse_sequence(s: &str) -> Result<Rule, String> {
    let mut items = Vec::new();
    for item in s.split(" ") {
	match item.trim().parse() {
	    Ok(n) => {
		items.push(n);
	    }
	    Err(e) => {
		return Err(format!("failed to parse item '{}' of sequence '{}': {}",
				   item, s, e));
	    }
	}
    }
    if items.len() < 1 {
	Err("empty sequences are not allowed".to_string())
    } else {
	Ok(Rule::Sequence(items))
    }
}

fn parse_alternative(s: &str) -> Result<Rule, String> {
    if let Some(caps) = ALT_RE.captures(s) {
	let left = seq_items(&caps[1].trim())?;
	let right = seq_items(&caps[2].trim())?;
	Ok(Rule::Alternative(left, right))
    } else {
	Err(format!("'{}' is not an alternative rule", s))
    }
}

fn parse_literal(s: &str) -> Result<Rule, String> {
    if let Some(caps) = LIT_RE.captures(s) {
	let ch = caps[1].chars().next().expect("LIT_RE should select a char");
	Ok(Rule::Literal(ch))
    } else {
	Err(format!("'{}' is not a literal rule", s))
    }
}

fn parse_line(s: &str) -> Result<(RuleId, Rule), String> {
    let (rule_id, body) = match RULE_RE.captures(s) {
	None => {
	    return Err(format!("line does not look like a rule definition: '{}'", s));
	}
	Some(caps) => match caps[1].parse::<RuleId>() {
	    Err(e) => {
		return Err(format!("failed to parse rule id '{}': {}", &caps[1], e));
	    }
	    Ok(id) => (id, caps[2].to_string()),
	}
    };
    let funcs = [parse_alternative, parse_sequence, parse_literal];
    for f in &funcs {
	match f(&body) {
	    Ok(def) => {
		return Ok((rule_id, def));
	    },
	    Err(_) => (),
	}
    }
    Err(format!("failed to parse rule body '{}'", body))
}

fn read_lines() -> Result<Vec<String>, String> {
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
    Ok(input_lines)
}

fn run() -> Result<(), String> {
    let lines = read_lines()?;
    let mut rules: HashMap<RuleId, Rule> = HashMap::new();
    let mut messages: Vec<String> = Vec::new();
    let mut saw_blank = false;
    for line in lines {
	if saw_blank {
	    messages.push(line);
	} else {
	    if line.is_empty() {
		saw_blank = true;
	    } else {
		let (id, rule) = parse_line(&line)?;
		rules.insert(id, rule);
	    }
	}
    }
    for (id, r) in rules {
	println!("rule {}: {}", id, r);
    }
    for msg in messages {
	println!("check: {}", msg);
    }
    Ok(())
}

fn self_test() {
    assert_eq!(parse_literal(" \"x\""),
	       Ok(Rule::Literal('x')));
    assert_eq!(parse_alternative(" 1 2 | 9 43"),
	       Ok(Rule::Alternative(vec![1,2],
				    vec![9,43])));
}


fn main() {
    self_test();
    std::process::exit(match run() {
	Ok(_) => 0,
	Err(err) => {
	    eprintln!("error: {}", err);
	    1
	}
    });
}
