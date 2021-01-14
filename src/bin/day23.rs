extern crate log;
extern crate pretty_env_logger;
use std::fmt;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Read;

use std::io;

#[derive(Debug)]
struct Cup {
    label: u32,
    succ: usize,
}

impl fmt::Display for Cup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
	write!(f, "{}", self.label)
    }
}

#[derive(Debug)]
struct CupCircle {
    cups: Vec<Cup>,
    label_to_pos: Vec<usize>,
    current_pos: usize,
}

impl CupCircle {
    fn new(v: &Vec<u32>) -> CupCircle {
	if v.is_empty() {
	    panic!("CupCircle::new cannot accept an empty Vec");
	}
	CupCircle{
	    current_pos: 0,
	    cups: (0..v.len())
		.map(|i| Cup{
		    label: v[i] as u32,
		    succ: if i >= v.len() { 0 } else { i + 1 },
		})
		.collect(),
	    label_to_pos: itertools::sorted(
		v.iter().enumerate()
		    .map(|(pos, label)| (*label, pos)))
		.into_iter()
		.map(|(_, pos)| pos)
		.collect(),
	}
    }

    fn label_of_current(&self) -> u32 {
	self.cups[self.current_pos].label
    }

    fn check(&self) {
	let mut labels_seen: HashSet<u32> = HashSet::new();
	let mut succ_seen: HashSet<usize> = HashSet::new();
	for c in &self.cups {
	    if labels_seen.contains(&c.label) {
		panic!(format!("duplicate label {}", c.label));
	    }
	    labels_seen.insert(c.label);
	    if succ_seen.contains(&c.succ) {
		panic!(format!("duplicate successor {}", c.succ));
	    }
	    succ_seen.insert(c.succ);
	}
	drop(labels_seen);
	drop(succ_seen);

	let mut pos_seen: HashSet<usize> = HashSet::new();
	for pos in &self.label_to_pos {
	    if pos_seen.contains(&pos) {
		panic!(format!("duplicate position {}", pos));
	    }
	    pos_seen.insert(*pos);
	    assert_eq!(*pos, self.get_pos(self.cups[*pos].label));
	}
	drop(pos_seen);
    }

    fn get_pos(&self, label: u32) -> usize {
	let lab = label as usize;
	assert!(label > 0);
	assert!((lab - 1) < self.label_to_pos.len());
	return self.label_to_pos[lab-1];
    }

    fn extend(&mut self, want: u32) {
	let mut max = match self.cups.iter().map(|c| c.label).max() {
	    Some(n) => n,
	    None => {
		if want > 0 {
		    self.cups.push(Cup{
			label: 1,
			succ: 0,
		    });
		    self.label_to_pos.push(0);
		    1
		} else {
		    return;
		}
	    }
	};
	self.cups.iter_mut().rev().next().unwrap().succ = self.cups.len();
	self.cups.reserve(want as usize);
	self.label_to_pos.reserve(want as usize);
	while want > max {
	    max += 1;
	    let here = self.cups.len();
	    self.cups.push(
		Cup{
		    label: max,
		    succ: here + 1,
		}
	    );
	    self.label_to_pos.push(here);
	}
	self.cups.iter_mut().rev().next().unwrap().succ = 0;
    }
}

impl fmt::Display for CupCircle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
	let current = self.label_of_current();
	let mut prefix = "";
	for c in self {
	    if c == current {
		write!(f, "{}({})", prefix, c)?;
	    } else {
		write!(f, "{}{}", prefix, c)?;
	    }
	    prefix = " ";
	}
	Ok(())
    }
}



struct CupCircleIter<'r> {
    pos: Option<usize>,
    circle: &'r CupCircle,
}

impl Iterator for CupCircleIter<'_> {
    type Item=u32;

    fn next(&mut self) -> Option<Self::Item> {
	match self.pos {
	    None => {
		None
	    }
	    Some(p) => {
		self.pos = match self.circle.cups[p].succ {
		    0 => None,
		    next => Some(next),
		};
		Some(self.circle.cups[p].label)
	    }
	}
    }
}

impl<'a> IntoIterator for &'a CupCircle {
    type Item = u32;
    type IntoIter = CupCircleIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
	CupCircleIter{
	    pos: if self.cups.is_empty() { None } else { Some(0) },
	    circle: &self,
	}
    }
}



fn show(label: &str, cups: &CupCircle) {
    println!("Part 1: {} cups are:", label);
    for (i, c) in cups.cups.iter().enumerate() {
	println!("{:>2}: {:?} succ={} (label_to_pos[{}]={})",
		 i, c.label, c.succ,
		 c, cups.get_pos(c.label));
    }
}

fn part1(initial: &Vec<u32>) -> Result<(), String>
{
    let mut cups = CupCircle::new(initial);
    show("initial", &cups);
    cups.check();
    cups.extend(20);
    show("extended", &cups);
    cups.check();
    println!("Part 1: cups: {}", cups);
    Ok(())
}

fn run() -> Result<(), String> {
    let mut buffer = String::new();
    match io::stdin().read_to_string(&mut buffer) {
	Ok(_) => (),
	Err(e) => { return Err(format!("I/O error: {}", e)); }
    };
    let labels: Vec<u32> = buffer.trim().chars()
	.map(|ch| {
	    match ch.to_digit(10) {
		None => {
		    panic!(format!("labels must be valid digits but '{}' is not", ch));
		}
		Some(d) => d,
	    }
	})
	.collect();
    part1(&labels)?;
    Ok(())
}

fn main() {
    // the env logger is configured with $RUST_LOG.
    // For example RUST_LOG=debug day20
    pretty_env_logger::init();
    std::process::exit(match run() {
	Ok(_) => 0,
	Err(err) => {
	    eprintln!("error: {}", err);
	    1
	}
    });
}
