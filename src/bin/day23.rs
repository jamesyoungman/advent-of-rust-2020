extern crate log;
extern crate pretty_env_logger;
use std::fmt;
use std::collections::HashSet;
use std::io::Read;

use std::io;

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Clone,Copy,Hash)]
enum Check {
    Always,
    Never
}

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

struct CupCircle {
    verbose: bool,
    checks: Check,
    cups: Vec<Cup>,
    label_to_pos: Vec<usize>,
    current_pos: usize,
    max_label: u32,
}

impl fmt::Debug for CupCircle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
	write!(f, "CupCircle{{\nverbose: {}\nchecks: {:?}\ncups: [\n",
	       self.verbose, self.checks)?;
	for i in 0..self.cups.len() {
	    write!(f, "  [{}] = {:?}", i, self.cups[i])?;
	}
	f.write_str("]\nlabel_to_pos: [\n")?;
	for i in 0..self.label_to_pos.len() {
	    write!(f, "  [{}] for {} = {:?}", i, i+1, self.label_to_pos[i])?;
	}
	write!(f, "]\ncurrent_pos={}, max_label={}\n}}", self.current_pos, self.max_label)
    }
}

impl CupCircle {
    fn new(verbose: bool, checks: Check, v: &Vec<u32>) -> CupCircle {
	if v.is_empty() {
	    panic!("CupCircle::new cannot accept an empty Vec");
	}
	CupCircle{
	    verbose,
	    checks,
	    current_pos: 0,
	    cups: (0..v.len())
		.map(|i| Cup{
		    label: v[i] as u32,
		    succ: if i + 1 >= v.len() { 0 } else { i + 1 },
		})
		.collect(),
	    label_to_pos: itertools::sorted(
		v.iter().enumerate()
		    .map(|(pos, label)| (*label, pos)))
		.into_iter()
		.map(|(_, pos)| pos)
		.collect(),
	    max_label: *v.iter().max().expect("Vec should not be empty"),
	}
    }

    fn label_of_current(&self) -> u32 {
	self.cups[self.current_pos].label
    }

    fn play(&mut self, move_number: usize) {
	if self.verbose {
	    println!("\n-- move {} --\ncups: {}", move_number, self);
	}
	self.check();
	 // remove 3 cups.
	let pos1 = self.cups[self.current_pos].succ;
	let pos2 = self.cups[pos1].succ;
	let pos3 = self.cups[pos2].succ;
	if self.verbose {
	    println!("pick up: {}, {}, {}",
		     self.cups[pos1].label,
		     self.cups[pos2].label,
		     self.cups[pos3].label);
	}
	self.cups[self.current_pos].succ = self.cups[pos3].succ;
	 // select the destination cup.
	let mut dest_label = self.cups[self.current_pos].label - 1;
	if dest_label < 1 {
	    dest_label = self.max_label;
	}
	while dest_label == self.cups[pos1].label
	    || dest_label == self.cups[pos2].label
	    || dest_label == self.cups[pos3].label {
	    dest_label -= 1;
	    if dest_label < 1 {
		dest_label = self.max_label;
	    }
	}
	let dest_pos = self.get_pos(dest_label);
	if self.verbose {
	    println!("destination: {} (at position {})", dest_label, dest_pos);
	}
	// Splice the 3 taken cups back in immediately after the
	// destination cup.
	let tail_pos = self.cups[dest_pos].succ;
	self.cups[pos3].succ = tail_pos;
	self.cups[dest_pos].succ = pos1;
	// Select a new current cup.
	self.current_pos = self.cups[self.current_pos].succ;
	assert!(self.current_pos < self.cups.len());
	self.check();
    }

    fn check(&self) {
	if self.checks == Check::Never {
	    return;
	}
	if self.current_pos >= self.cups.len() {
	    panic!(format!("current_pos {} is out-of-range", self.current_pos));
	}
	if self.max_label as usize != self.cups.len() {
	    panic!(format!("max_label {} is unexpected; should be {}",
			   self.max_label, self.cups.len()));
	}
	let mut labels_seen: HashSet<u32> = HashSet::new();
	let mut succ_seen: HashSet<usize> = HashSet::new();
	for (i, c) in self.cups.iter().enumerate() {
	    if c.succ >= self.cups.len() {
		panic!(format!("cup {} has out-of-range succ value {}", i, c.succ));
	    }
	    if c.label < 1 || c.label as usize > self.cups.len() {
		panic!(format!("cup {} has out-of-range label value {}", i, c.label));
	    }
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
	assert!(!self.cups.is_empty());
	self.cups.iter_mut().rev().next().unwrap().succ = self.cups.len();
	self.cups.reserve(want as usize);
	self.label_to_pos.reserve(want as usize);
	while want > self.max_label {
	    self.max_label += 1;
	    let here = self.cups.len();
	    self.cups.push(
		Cup{
		    label: self.max_label,
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
		if p >= self.circle.cups.len() {
		    panic!(format!("CupCircleIter: pos {} out of range:\n{:?}",
				   p, self.circle));
		}
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
	self.check();
	CupCircleIter{
	    pos: if self.cups.is_empty() { None } else { Some(0) },
	    circle: &self,
	}
    }
}



fn show(part: u32, label: &str, cups: &CupCircle) {
    println!("Part {}: {} cups are:", part, label);
    for (i, c) in cups.cups.iter().enumerate() {
	println!("{:>2}: {:?} succ={} (label_to_pos[{}]={})",
		 i, c.label, c.succ,
		 c, cups.get_pos(c.label));
    }
}

fn cups_succ(label: u32, n: usize, cups: &CupCircle) -> u32 {
    let mut pos = cups.get_pos(label);
    for _ in 0..n {
	pos = cups.cups[pos].succ;
    }
    cups.cups[pos].label
}


fn play_moves(count: usize, cups: &mut CupCircle) {
    for move_number in 1..=count {
	cups.play(move_number);
    }
}

fn part1(initial: &Vec<u32>) -> Result<(), String>
{
    let mut cups = CupCircle::new(true, Check::Always, initial);
    show(1, "initial", &cups);
    cups.check();
    play_moves(100, &mut cups);
    if cups.verbose {
	println!("\n-- final --\ncups: {}", cups);
    }
    print!("Part 1: labels after 1: ");
    let mut pos: usize = cups.cups[cups.get_pos(1)].succ as usize;
    for _ in 0..(cups.cups.len()-1) {
	let cup = &cups.cups[pos as usize];
	print!("{}", cup.label);
	pos = cup.succ as usize;
    }
    println!("");
    Ok(())
}

fn part2(initial: &Vec<u32>) -> Result<(), String>
{
    let mut cups = CupCircle::new(false, Check::Never, initial);
    show(2, "initial (before extending)", &cups);
    cups.extend(1000000);
    play_moves(10 * 1000 * 1000, &mut cups);
    let succ1 = cups_succ(1, 1, &cups);
    let succ2 = cups_succ(1, 2, &cups);
    println!("Part 2: product is {} * {} = {}",
	     succ1, succ2, (succ1 as usize) * (succ2 as usize));
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
    part2(&labels)?;
    Ok(())
}

fn main() {
    // the env logger is configured with $RUST_LOG.
    // For example RUST_LOG=debug day23
    pretty_env_logger::init();
    std::process::exit(match run() {
	Ok(_) => 0,
	Err(err) => {
	    eprintln!("error: {}", err);
	    1
	}
    });
}
