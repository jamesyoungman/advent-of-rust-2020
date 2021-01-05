use std::collections::HashSet;
use std::ops::RangeInclusive;
use std::io::{self, Read};
use std::fmt;

type Ordinate = i32;
type OrdinateRange = RangeInclusive<i32>;

#[derive(Copy,Clone,PartialEq,Eq,PartialOrd,Ord,Hash)]
struct Pos4
{
    x: Ordinate,
    y: Ordinate,
    z: Ordinate,
    w: Ordinate,
}

struct Lattice {
    xrange: OrdinateRange,
    yrange: OrdinateRange,
    zrange: OrdinateRange,
    wrange: OrdinateRange,
    active: HashSet<Pos4>
}

fn range_size(r: &OrdinateRange) -> usize {
    if r.end() < r.start() {
	0
    } else {
	let result: i32 = r.end() - r.start() + 1;
	assert!(result > 0);
	result as usize
    }
}

impl Lattice {
    fn slice_as_str(&self, w: Ordinate, z: Ordinate) -> String {
	let mut output = String::with_capacity(
	    range_size(&self.xrange) * range_size(&self.yrange));
	for y in self.yrange.clone() {
	    for x in self.xrange.clone() {
		let pos = Pos4{x, y, z, w};
		output.push(if self.active.contains(&pos) {'#'} else {'.'})
	    }
	    output.push('\n');
	}
	output
    }

    fn from_string(s: &str) -> Result<Lattice, String> {
	let mut xrange: OrdinateRange = 0..=0;
	let mut yrange: OrdinateRange = 0..=0;
	let mut x = 0;
	let mut y = 0;
	let mut cells: HashSet<Pos4> = HashSet::new();
	for ch in s.chars() {
	    match ch {
		'#' => {
		    cells.insert(Pos4{x, y, z: 0, w: 0});
		    if &x > xrange.end() {
			xrange = 0..=x;
		    }
		    if &y > yrange.end() {
			yrange = 0..=y;
		    }
		    x += 1;
		}
		'.' => {
		    x += 1;
		}
		'\n' => {
		    x = 0;
		    y += 1;
		}
		_ => {
		    return Err(format!("unexpected character '{}' in input", ch));
		}
	    }
	}
	Ok(Lattice{
	    xrange,
	    yrange,
	    zrange: 0..=0,
	    wrange: 0..=0,
	    active: cells,
	})
    }
}

impl fmt::Display for Lattice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
	for w in self.wrange.clone() {
	    for z in self.zrange.clone() {
		match write!(f, "z={},w={}\n{}", z, w, self.slice_as_str(w, z)) {
		    Err(e) => { return Err(e); }
		    _ => ()
		}
	    }
	}
	Ok(())
    }
}

fn read_input() -> Result<Lattice, String> {
    let mut buffer = String::new();
    match io::stdin().read_to_string(&mut buffer) {
	Ok(_) => Lattice::from_string(&buffer),
	Err(e) => Err(format!("I/O error: {}", e))
    }
}

fn run() -> Result<(), String> {
    let initial = read_input()?;
    println!("Initial state is:\n{}", initial);
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
