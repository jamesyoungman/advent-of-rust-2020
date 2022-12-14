extern crate itertools;
extern crate lazy_static;
extern crate regex;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::cmp;
use std::collections::HashMap;
use std::fmt;
use std::io;
use std::io::BufRead;

lazy_static! {
    static ref RULE_RE: Regex = Regex::new(r"^(\d+): (.*)$").expect("RULE_RE");
    static ref LIT_RE: Regex = Regex::new("\"(.)\"$").expect("LIT_RE");
    static ref ALT_RE: Regex = Regex::new(r"([^|]+)[|]([^|]+)$").expect("ALT_RE");
}

type RuleId = i32;

#[derive(PartialEq, Debug)]
enum Rule {
    Sequence(Vec<RuleId>),                 // e.g. [28: 16 1]
    Alternative(Vec<RuleId>, Vec<RuleId>), // e.g. [26: 14 22 | 1 20]
    Literal(char),                         // e.g. [14: "b"]
    CannedRegex(String),                   // used for part 2.
}

fn make_group(pattern: &str) -> String {
    // By using non-capturing groups we save memory and compute when
    // performing pattern matching (and, probably, memory when
    // compiling the regex).
    format!("(:?{})", pattern)
}

fn repeat(s: &str, n: usize) -> String {
    let mut result = String::with_capacity(n * s.len());
    for _ in 0..n {
        result.push_str(s);
    }
    result
}

fn balanced(left: &str, right: &str, maxlen: usize) -> String {
    let mut tmp: Vec<String> = Vec::new();
    for reps in 1..=maxlen {
        tmp.push(make_group(
            format!("{}{}", repeat(left, reps), repeat(right, reps)).as_str(),
        ));
    }
    make_group(&tmp.iter().join("|"))
}

fn translate_rule_sequence(
    items: &[RuleId],
    rules: &HashMap<RuleId, Rule>,
) -> Result<String, String> {
    items
        .iter()
        .map(|i| translate_to_regex_pattern(i, rules))
        .collect()
}

fn translate_alternative(
    left_items: &[RuleId],
    right_items: &[RuleId],
    rules: &HashMap<RuleId, Rule>,
) -> Result<String, String> {
    let lpat = translate_rule_sequence(left_items, rules)?;
    let rpat = translate_rule_sequence(right_items, rules)?;
    Ok(if lpat.len() != 1 || rpat.len() != 1 {
        make_group(&format!("{}|{}", lpat, rpat))
    } else {
        format!("[{}{}]", lpat, rpat)
    })
}

fn translate_to_regex_pattern(
    start_rule: &RuleId,
    rules: &HashMap<RuleId, Rule>,
) -> Result<String, String> {
    match rules.get(start_rule) {
        None => Err(format!("missing definition for rule {}", start_rule)),
        Some(Rule::Sequence(items)) => translate_rule_sequence(items, rules),
        Some(Rule::Alternative(left_items, right_items)) => {
            translate_alternative(left_items, right_items, rules)
        }
        Some(Rule::CannedRegex(pattern)) => Ok(pattern.to_string()),
        Some(Rule::Literal(ch)) => Ok(ch.to_string()),
    }
}

fn seq_items(s: &str) -> Result<Vec<RuleId>, String> {
    let mut result = Vec::new();
    for field in s.split(' ') {
        match field.parse() {
            Ok(n) => {
                result.push(n);
            }
            Err(e) => {
                return Err(format!("failed to parse integer '{}': {}", field, e));
            }
        }
    }
    Ok(result)
}

fn seq_fmt(seq: &[RuleId]) -> String {
    itertools::join(seq.iter(), " ")
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Rule::Sequence(idlist) => f.write_str(&seq_fmt(idlist)),
            Rule::Alternative(left, right) => write!(f, "{} | {}", seq_fmt(left), seq_fmt(right)),
            Rule::Literal(ch) => write!(f, "\"{}\"", ch),
            Rule::CannedRegex(pattern) => write!(f, "[canned] {}", pattern),
        }
    }
}

fn parse_sequence(s: &str) -> Result<Rule, String> {
    let mut items = Vec::new();
    for item in s.split(' ') {
        match item.trim().parse() {
            Ok(n) => {
                items.push(n);
            }
            Err(e) => {
                return Err(format!(
                    "failed to parse item '{}' of sequence '{}': {}",
                    item, s, e
                ));
            }
        }
    }
    if items.is_empty() {
        Err("empty sequences are not allowed".to_string())
    } else {
        Ok(Rule::Sequence(items))
    }
}

fn parse_alternative(s: &str) -> Result<Rule, String> {
    if let Some(caps) = ALT_RE.captures(s) {
        let left = seq_items(caps[1].trim())?;
        let right = seq_items(caps[2].trim())?;
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
            return Err(format!(
                "line does not look like a rule definition: '{}'",
                s
            ));
        }
        Some(caps) => match caps[1].parse::<RuleId>() {
            Err(e) => {
                return Err(format!("failed to parse rule id '{}': {}", &caps[1], e));
            }
            Ok(id) => (id, caps[2].to_string()),
        },
    };
    let funcs = [parse_alternative, parse_sequence, parse_literal];
    for f in &funcs {
        if let Ok(def) = f(&body) {
            return Ok((rule_id, def));
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
    }
    Ok(input_lines)
}

fn count_matches(rx: &Regex, messages: &[String], show_matches: &bool) -> usize {
    let printer = |m: &str| {
        if *show_matches {
            println!("{}", m);
        }
    };
    messages
        .iter()
        .filter(|m| rx.is_match(m))
        .inspect(|s| printer(s.as_str()))
        .count()
}

fn compile_regex(pattern: &str) -> Regex {
    let anchored = format!("^{}$", make_group(pattern));
    Regex::new(&anchored).expect("failed to compile regex")
}

fn run() -> Result<(), String> {
    let show_matches = false;
    let show_patterns = false;
    let lines = read_lines()?;
    let mut rules: HashMap<RuleId, Rule> = HashMap::new();
    let mut messages: Vec<String> = Vec::new();
    let mut saw_blank = false;
    for line in lines {
        if saw_blank {
            messages.push(line);
        } else if line.is_empty() {
            saw_blank = true;
        } else {
            let (id, rule) = parse_line(&line)?;
            rules.insert(id, rule);
        }
    }

    let pat1 = translate_to_regex_pattern(&0, &rules)?;
    if show_patterns {
        println!("Part 1: regex for 0 is {}", pat1);
    }
    let rx1 = compile_regex(&pat1);

    let maxlen: usize = match messages.iter().map(|m| m.len()).max() {
        None => {
            println!("No messages, nothing to do");
            return Ok(());
        }
        Some(n) => n,
    };

    println!(
        "Part 1: {} matches",
        count_matches(&rx1, &messages, &show_matches)
    );

    // Customisations for part 2.
    let rule31_pattern = translate_to_regex_pattern(&31, &rules)?;
    let rule42_pattern = translate_to_regex_pattern(&42, &rules)?;
    rules.insert(8, Rule::CannedRegex(format!("(({})+)", rule42_pattern)));
    // Rule 11 should match XY, XXYY, XXXYYY, ... without limit, which
    // cannot be represented in a regex, so we have to choose an upper
    // limit.  The obvious upper limit is the maximum message length,
    // but if we choose that, the Rust Regex implementation will
    // refuse to compile the pattern.  At lower levels (e.g. 10), the
    // implementation will try but run out of memory or take a long
    // time.  Hence we determined experimentally that a maximum repeat
    // count of 5 gets us the right answer.
    let repeats = cmp::min(maxlen, 5);
    rules.insert(
        11,
        Rule::CannedRegex(balanced(&rule42_pattern, &rule31_pattern, repeats)),
    );

    let pat2 = translate_to_regex_pattern(&0, &rules)?;
    if show_patterns {
        println!("Part 2: regex for 0 is {}", pat2);
    }
    let rx2 = compile_regex(&pat2);
    println!(
        "Part 2: {} matches",
        count_matches(&rx2, &messages, &show_matches)
    );

    Ok(())
}

fn self_test() {
    assert_eq!(parse_literal(" \"x\""), Ok(Rule::Literal('x')));
    assert_eq!(
        parse_alternative(" 1 2 | 9 43"),
        Ok(Rule::Alternative(vec![1, 2], vec![9, 43]))
    );
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
