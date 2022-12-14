use std::collections::BTreeSet;
use std::io;
use std::io::BufRead;

extern crate itertools;

fn union(s1: String, s2: &str) -> String {
    let u2: BTreeSet<char> = s2.chars().collect();
    s1.chars().collect::<BTreeSet<char>>().union(&u2).collect()
}

fn intersection(s1: String, s2: &str) -> String {
    let u2: BTreeSet<char> = s2.chars().collect();
    s1.chars()
        .collect::<BTreeSet<char>>()
        .intersection(&u2)
        .collect()
}

// Each person is represented by a String containing their unique answers.
// Each group is a Vec<String> containing the answers of each person in the group.
// Hence the input - which is all the groups - is represented by a Vec<Vec<String>>.
fn read_input(reader: impl BufRead) -> Result<Vec<Vec<String>>, io::Error> {
    let empty_string = "".to_string();
    let mut current_group = Vec::new();
    let mut result = Vec::new();
    for line_or_fail in reader.lines() {
        match line_or_fail {
            Ok(line) => {
                if line.is_empty() {
                    // Groups are separated by a blank line.
                    result.push(current_group);
                    current_group = Vec::new();
                } else {
                    current_group.push(union(line, &empty_string));
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    if !current_group.is_empty() {
        result.push(current_group);
    }
    Ok(result)
}

fn count_anyone(g: &[String]) -> usize {
    g.iter().fold(String::new(), |acc, s| union(acc, s)).len()
}

fn count_everyone(g: &[String]) -> usize {
    let mut it = g.iter();
    match it.next() {
        Some(s) => it
            .fold(s.to_string(), |acc, s| intersection(acc, s.as_str()))
            .len(),
        None => 0,
    }
}

fn run() -> Result<(), std::io::Error> {
    let groups: Vec<Vec<String>> = read_input(io::BufReader::new(io::stdin()))?;
    println!("There are a total of {} groups in the input", groups.len());
    println!(
        "Part 1: {}",
        groups.iter().fold(0, |total, g| total + count_anyone(g))
    );
    println!(
        "Part 2: {}",
        groups.iter().fold(0, |total, g| total + count_everyone(g))
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
