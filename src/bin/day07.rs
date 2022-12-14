extern crate lazy_static;
extern crate regex;

use std::collections::HashMap;
use std::collections::HashSet;
use std::io;
use std::io::BufRead;

use lazy_static::lazy_static; // 1.3.0
use regex::Regex;

lazy_static! {
    static ref CONTAINS_BAG_RE: Regex = Regex::new(r" ?(\d*) (.*) bags?").unwrap();
    static ref CONTAINS_RE: Regex = Regex::new(r"((.*),?)*").unwrap();
    static ref EMPTY_RE: Regex = Regex::new(r"no other bags$").unwrap();
    static ref LINE_RE: Regex = Regex::new(r"^(.*) bags contain (.*)$").unwrap();
}

#[derive(Debug)]
struct Bag {
    required_children: HashMap<String, u64>,
    allowed_parents: HashSet<String>,
}

impl Bag {
    fn new(_col: &str) -> Bag {
        Bag {
            required_children: HashMap::new(),
            allowed_parents: HashSet::new(),
        }
    }

    fn add_required_child(&mut self, colour: &str, quantity: &u64) {
        self.required_children.insert(colour.to_string(), *quantity);
    }

    fn add_allowed_parent(&mut self, colour: &str) {
        self.allowed_parents.insert(colour.to_string());
    }
}

struct BagDefs {
    definitions: HashMap<String, Bag>,
}

impl BagDefs {
    fn maybe_add_bag(&mut self, colour: &str) {
        if !self.definitions.contains_key(colour) {
            self.definitions
                .insert(colour.to_string(), Bag::new(colour));
        }
    }

    fn get_or_add_bag(&mut self, colour: &str) -> &mut Bag {
        self.maybe_add_bag(colour);
        self.definitions.get_mut(colour).unwrap()
    }

    fn add_bag(&mut self, parent_colour: &str, child_colour: &str, quantity: &u64) {
        let parent: &mut Bag = self.get_or_add_bag(parent_colour);
        parent.add_required_child(child_colour, quantity);

        let child: &mut Bag = self.get_or_add_bag(child_colour);
        child.add_allowed_parent(&parent_colour.to_string());
    }

    fn new() -> BagDefs {
        BagDefs {
            definitions: HashMap::new(),
        }
    }

    fn can_contain(&self, parent_colour: &str, wanted: &str) -> bool {
        let parent = match self.definitions.get(parent_colour) {
            None => {
                panic!("can_contain: unknown parent {}", parent_colour);
            }
            Some(parent) => parent,
        };
        if parent.required_children.contains_key(wanted) {
            return true;
        }
        for child_colour in parent.required_children.keys() {
            if self.can_contain(child_colour, wanted) {
                return true;
            }
        }
        return false;
    }

    fn possible_parents(&self, colour_wanted: &str) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        for parent_colour in self.definitions.keys() {
            if self.can_contain(parent_colour, colour_wanted) {
                result.push(parent_colour.to_string())
            }
        }
        result
    }

    fn count_children(&self, parent_colour: &str) -> u64 {
        match self.definitions.get(parent_colour) {
            Some(bag) => bag
                .required_children
                .iter()
                .fold(0, |acc, (colour, children)| {
                    acc + children * (1 + self.count_children(colour))
                }),
            None => {
                panic!(
                    "we know nothing about what goes into {} bags",
                    parent_colour
                );
            }
        }
    }
}

fn parse_line(orig_line: &str) -> (String, HashMap<String, u64>) {
    let line = orig_line.trim_end_matches(".");
    let cap = match LINE_RE.captures(line) {
        None => {
            panic!("malformed line '{}'", line);
        }
        Some(cap) => cap,
    };
    let parent_colour = (&cap[1]).to_string();
    let contents_str = &cap[2];
    if EMPTY_RE.is_match(contents_str) {
        return (parent_colour.to_string(), HashMap::new());
    }
    let items: Vec<&str> = contents_str.split(",").collect();
    if items.is_empty() {
        panic!(
            "every non-empty bag should have contents: '{}' does not",
            contents_str
        );
    }
    let mut contents: HashMap<String, u64> = HashMap::new();
    for item in items.iter() {
        if EMPTY_RE.is_match(contents_str) {
            break;
        }
        let cap = match CONTAINS_BAG_RE.captures(item) {
            None => {
                panic!("content item {} should match CONTAINS_BAG_RE", item);
            }
            Some(cap) => cap,
        };
        contents.insert((&cap[2]).to_string(), cap[1].parse().unwrap());
    }
    (parent_colour, contents)
}

fn run() -> Result<(), std::io::Error> {
    let mut definitions = BagDefs::new();
    for thing in io::BufReader::new(io::stdin()).lines() {
        match thing {
            Ok(line) => {
                let (parent_colour, contents) = parse_line(&line);
                for (child_colour, quantity) in contents.iter() {
                    definitions.add_bag(&parent_colour, &child_colour, quantity);
                }
            }
            Err(e) => return Err(e),
        }
    }
    //for (colour, bag) in definitions.definitions.iter() {
    //	println!("{}: {:?}", colour, bag);
    //}
    let sg = "shiny gold";
    let parents = definitions.possible_parents(sg);
    println!(
        "Part 1: a {} bag might be contained in {} bags: {}",
        sg,
        parents.len(),
        parents.join(", ")
    );
    println!(
        "Part 2: a {} bag contains a total of {} other bags",
        sg,
        definitions.count_children(sg)
    );
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
