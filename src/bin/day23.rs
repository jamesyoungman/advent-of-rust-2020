extern crate log;
extern crate pretty_env_logger;
use std::fmt;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Read;

use std::io;

#[derive(Debug)]
struct CupCircle {
    labels: Vec<u32>,
    pos_to_succ: Vec<usize>,
    label_to_pos: Vec<usize>,
}

impl CupCircle {
    fn new(v: &Vec<u32>) -> CupCircle {
	CupCircle{
	    labels: (0..v.len()).map(|i| v[i] as u32).collect(),
	    pos_to_succ: (0..v.len())
		.map(|i| {
		    if i >= v.len()
		    {
			0
		    } else {
			i + 1
		    }
		} as usize)
		.collect(),
	    label_to_pos: itertools::sorted(
		v.iter().enumerate()
		    .map(|(pos, label)| (*label, pos)))
		.into_iter()
		.map(|(_, pos)| pos)
		.collect(),
	}
    }

    fn check(&self) {
	let mut labels_seen: HashSet<u32> = HashSet::new();
	for label in &self.labels {
	    if labels_seen.contains(&label) {
		panic!(format!("duplicate label {}", label));
	    }
	    labels_seen.insert(*label);
	}
	drop(labels_seen);

	let mut succ_seen: HashSet<usize> = HashSet::new();
	for succ in &self.pos_to_succ {
	    if succ_seen.contains(&succ) {
		panic!(format!("duplicate successor {}", succ));
	    }
	    succ_seen.insert(*succ);
	}
	drop(succ_seen);

	let mut pos_seen: HashSet<usize> = HashSet::new();
	for pos in &self.label_to_pos {
	    if pos_seen.contains(&pos) {
		panic!(format!("duplicate position {}", pos));
	    }
	    pos_seen.insert(*pos);
	    assert_eq!(*pos, self.get_pos(self.labels[*pos]));
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
	let mut max = match self.labels.iter().max() {
	    Some(n) => *n,
	    None => {
		if want > 0 {
		    self.labels.push(1);
		    self.pos_to_succ.push(0);
		    self.label_to_pos.push(0);
		    1
		} else {
		    return;
		}
	    }
	};
	*self.pos_to_succ.iter_mut().rev().next().unwrap() = self.pos_to_succ.len();
	self.labels.reserve(want as usize);
	self.pos_to_succ.reserve(want as usize);
	self.label_to_pos.reserve(want as usize);
	while want > max {
	    max += 1;
	    let here = self.labels.len();
	    self.labels.push(max);
	    self.pos_to_succ.push(here + 1);
	    self.label_to_pos.push(here);
	}
	*self.pos_to_succ.iter_mut().rev().next().unwrap() = 0;
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
		self.pos = match self.circle.pos_to_succ[p] {
		    0 => None,
		    next => Some(next),
		};
		Some(self.circle.labels[p])
	    }
	}
    }
}

impl<'a> IntoIterator for &'a CupCircle {
    type Item = u32;
    type IntoIter = CupCircleIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
	CupCircleIter{
	    pos: if self.labels.is_empty() { None } else { Some(0) },
	    circle: &self,
	}
    }
}



fn show(label: &str, cups: &CupCircle) {
    println!("Part 1: {} cups are:", label);
    for (i, c) in cups.labels.iter().enumerate() {
	println!("{:>2}: {:?} succ={} (label_to_pos[{}]={})",
		 i, c, cups.pos_to_succ[i],
		 c, cups.get_pos(*c));
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
    println!("Part 1: cups debug: {:?}", cups);
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
