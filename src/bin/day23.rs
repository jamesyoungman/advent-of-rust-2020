extern crate log;
extern crate pretty_env_logger;
use std::fmt;
use std::collections::HashMap;
use std::io::Read;

use std::io;

#[derive(Debug)]
struct Cup {
    label: u32,
    succ: usize,
}

#[derive(Debug)]
struct CupCircle {
    cups: Vec<Cup>,
    label_to_pos: Vec<usize>,
}

impl CupCircle {
    fn new(v: &Vec<u32>) -> CupCircle {
	let mut result = CupCircle{
	    cups: (0..v.len())
		.map(|i| Cup {
		    label: v[i] as u32,
		    succ: (if i >= v.len() { 0 } else { i + 1}) as usize,
		})
		.collect(),
	    label_to_pos: itertools::sorted(
		v.iter()
		    .enumerate()
		    .map(|(pos, label)| (*label, pos)))
		.into_iter()
		.map(|(_, pos)| pos)
		.collect(),
	};
	result
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
		    self.cups.push(Cup{label: 1, succ: 0});
		    1
		} else {
		    return;
		}
	    }
	};
	self.cups.iter_mut().rev().next().unwrap().succ = self.cups.len();
	self.cups.reserve(want as usize);
	while want > max {
	    self.cups.push(
		Cup{
		    label: max,
		    succ: (max + 1) as usize,
		}
	    );
	    max += 1;
	}
	self.cups.iter_mut().rev().next().unwrap().succ = 0;
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
		let here = &self.circle.cups[p];
		if here.succ == 0 {
		    self.pos = None
		} else {
		    self.pos = Some(here.succ);
		}
		Some(here.label)
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



fn part1(initial: &Vec<u32>) -> Result<(), String>
{
    let mut cups = CupCircle::new(initial);
    cups.extend(20);
    println!("Part 1: initial cups are:");
    for (i, c) in cups.cups.iter().enumerate() {
	println!("{:>2}: {:?}", i, c);
    }
    println!("Part 1: cupd debug: {:?}", cups);
    println!("Part 1: initial labels are: {}",
	     itertools::join(cups.into_iter(), " "));
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
