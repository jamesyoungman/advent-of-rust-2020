use std::collections::HashMap;
use std::collections::HashSet;
extern crate lazy_static;
extern crate regex;
use lazy_static::lazy_static;
use std::io;
use std::io::BufRead;
use std::string::String; // 1.3.0

use regex::Regex;

lazy_static! {
    static ref HEIGHT_RE: Regex = Regex::new(r"^([0-9]+)(cm|in)$").unwrap();
    static ref HAIR_COLOUR_RE: Regex = Regex::new(r"^#[0-9a-f]{6}").unwrap();
    static ref PID_RE: Regex = Regex::new("^[0-9]{9}$").unwrap();
    static ref VALID_EYE_COLOURS: HashSet<&'static str> =
        ["amb", "blu", "brn", "gry", "grn", "hzl", "oth"]
            .iter()
            .cloned()
            .collect();
}

struct Passport {
    attributes: HashMap<String, String>,
}

fn two_fields(delimiter: char, s: &str) -> Result<(String, String), &'static str> {
    let parts: Vec<&str> = s.split(delimiter).take(3).collect();
    if parts.len() == 2 {
        Ok((parts[0].to_string(), parts[1].to_string()))
    } else {
        Err("expected two fields")
    }
}

impl Passport {
    fn new(lines: &[String]) -> Passport {
        let mut h: HashMap<String, String> = HashMap::new();
        for setting in lines.iter().flat_map(|line| line.split_whitespace()) {
            match two_fields(':', setting) {
                Ok((attrib, value)) => {
                    h.insert(attrib, value);
                }
                Err(e) => {
                    panic!("{}", e);
                }
            }
        }
        Passport { attributes: h }
    }

    fn valid1(&self) -> bool {
        for attr in ["byr", "iyr", "eyr", "hgt", "hcl", "ecl", "pid"].iter() {
            if !self.attributes.contains_key(&attr.to_string()) {
                return false;
            }
        }
        true
    }

    fn attr_matches(&self, key: &str, rx: &Regex) -> bool {
        match self.attributes.get(&key.to_string()) {
            Some(value) => rx.is_match(value),
            None => false,
        }
    }

    fn valid_year(&self, key: &str, min_year: i32, max_year: i32) -> bool {
        match self.attributes.get(&key.to_string()) {
            Some(year_str) => match year_str.parse::<i32>() {
                Ok(y) => y >= min_year && y <= max_year,
                Err(_) => false,
            },
            None => false,
        }
    }

    fn valid_hair_colour(&self) -> bool {
        self.attr_matches("hcl", &HAIR_COLOUR_RE)
    }

    fn valid_eye_colour(&self) -> bool {
        match self.attributes.get(&"ecl".to_string()) {
            Some(c) => VALID_EYE_COLOURS.contains(c.as_str()),
            None => false,
        }
    }

    fn valid_height(&self) -> bool {
        let invalid = "".to_string();
        let value_str = self.attributes.get(&"hgt".to_string()).unwrap_or(&invalid);
        match HEIGHT_RE.captures(value_str) {
            Some(cap) => {
                if cap.len() != 3 {
                    return false;
                }
                match cap[1].parse::<i32>() {
                    Ok(n) => match &cap[2] {
                        "cm" => (150..=193).contains(&n),
                        "in" => (59..=76).contains(&n),
                        _ => false,
                    },
                    Err(_) => false,
                }
            }
            None => false,
        }
    }

    fn valid_passport_id(&self) -> bool {
        self.attr_matches("pid", &PID_RE)
    }

    fn valid2(&self) -> bool {
        self.valid1()
            && self.valid_year("byr", 1920, 2002)
            && self.valid_year("iyr", 2010, 2020)
            && self.valid_year("eyr", 2020, 2030)
            && self.valid_height()
            && self.valid_hair_colour()
            && self.valid_eye_colour()
            && self.valid_passport_id()
    }
}

fn part1(input: &[Passport]) -> usize {
    input.iter().filter(|p| p.valid1()).count()
}

fn part2(input: &[Passport]) -> usize {
    input.iter().filter(|p| p.valid2()).count()
}
fn read_input(reader: impl BufRead) -> Result<Vec<Passport>, io::Error> {
    let mut lines: Vec<String> = Vec::new();
    let mut result = Vec::new();
    for line_or_fail in reader.lines() {
        match line_or_fail {
            Ok(line) => {
                //println!("read_input: line is '{}'", line);
                if line.is_empty() {
                    result.push(Passport::new(&lines));
                    lines.clear();
                } else {
                    lines.push(line);
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    if !lines.is_empty() {
        result.push(Passport::new(&lines));
        lines.clear();
    }
    Ok(result)
}

fn run() -> Result<(), std::io::Error> {
    let passports = read_input(io::BufReader::new(io::stdin())).unwrap();
    println!(
        "There are a total of {} passports in the input",
        passports.len()
    );
    println!("Part 1: {} passports are valid", part1(&passports));
    println!("Part 2: {} passports are valid", part2(&passports));
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
