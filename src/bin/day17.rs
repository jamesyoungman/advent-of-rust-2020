use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::RangeInclusive;
use std::io::{self, Read};
use std::cmp::{min, max};
use std::fmt;

type Ordinate = i32;
type OrdinateRange = RangeInclusive<i32>;

#[derive(Debug,Copy,Clone,PartialEq,Eq,PartialOrd,Ord,Hash)]
struct Pos4
{
    x: Ordinate,
    y: Ordinate,
    z: Ordinate,
    w: Ordinate,
}

type Pos4Set = HashSet<Pos4>;
type Pos4Counter = HashMap<Pos4, usize>;

struct Lattice {
    xrange: OrdinateRange,
    yrange: OrdinateRange,
    zrange: OrdinateRange,
    wrange: OrdinateRange,
    active: Pos4Set,
    use_w: bool
}

fn update_range(n: &Ordinate, r: OrdinateRange) -> OrdinateRange {
    if r.contains(n) {
	r
    } else {
	min(*n, *r.start())..=max(*n, *r.end())
    }
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

impl Clone for Lattice {
    fn clone(&self) -> Lattice {
	Lattice{
	    xrange: self.xrange.clone(),
	    yrange: self.yrange.clone(),
	    zrange: self.zrange.clone(),
	    wrange: self.wrange.clone(),
	    active: self.active.clone(),
	    use_w: self.use_w,
	}
    }
}

impl Lattice {
    fn popcount(&self) -> usize {
	self.active.len()
    }

    fn slice_as_str(&self, w: Ordinate, z: Ordinate) -> String {
	let mut output = String::with_capacity(
	    (range_size(&self.xrange) + 1) * range_size(&self.yrange));
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
	let mut cells: Pos4Set = Pos4Set::new();
	for ch in s.chars() {
	    match ch {
		'#' => {
		    cells.insert(Pos4{x, y, z: 0, w: 0});
		    xrange = update_range(&x, xrange.clone());
		    yrange = update_range(&y, yrange.clone());
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
	    use_w: false,
	})
    }

    fn neighbours_of(&self, pos: &Pos4) -> Vec<Pos4> {
	let wrange = if self.use_w {
	    (pos.w-1)..=(pos.w+1)
	} else {
	    pos.w..=pos.w
	};
	let mut result = Vec::with_capacity(range_size(&wrange) * 3 * 3 * 3);
	for w in wrange {
	    for z in (pos.z-1)..=(pos.z+1) {
		for y in (pos.y-1)..=(pos.y+1) {
		    for x in (pos.x-1)..=(pos.x+1) {
			let neighbour = Pos4{w, z, y, x};
			if neighbour != *pos { // can't be my own neighbour
			    result.push(neighbour);
			}
		    }
		}
	    }
	}
	result
    }

    fn insert(&mut self, pos: Pos4) {
	if !self.use_w {
	    assert!(pos.w == 0)
	}
	self.active.insert(pos);
	self.xrange = update_range(&pos.x, self.xrange.clone());
	self.yrange = update_range(&pos.y, self.yrange.clone());
	self.zrange = update_range(&pos.z, self.zrange.clone());
	self.wrange = update_range(&pos.w, self.wrange.clone());
    }

    fn iterate(&self) -> Lattice {
	let mut neighbour_count: Pos4Counter = Pos4Counter::new();
	for neighbour in self.active.iter()
	    .flat_map(|pos| self.neighbours_of(&pos)) {
		neighbour_count.entry(neighbour)
		    .and_modify(|e| *e += 1)
		    .or_insert(1);
	    }

	let mut result = Lattice{
	    xrange: 0..=0,
	    yrange: 0..=0,
	    zrange: 0..=0,
	    wrange: 0..=0,
	    use_w: self.use_w,
	    active: Pos4Set::new(),
	};
	for p in itertools::chain(
	    // First, consider possible state changes in the cells
	    // that are currently active.
	    self.active.iter()
		.filter(|currently_active|
			match neighbour_count.get(&currently_active) {
			    Some(2) | Some(3) => true, // remains active
			    _ => false,	// becomes inactive
			}),
	    // Second, consider possible state changes in the cells
	    // that are not currently active.
	    neighbour_count.iter()
	    // The filter acceps only cells that are inactive but will become
	    // active.
		.filter(|(pos, num_neighbours)|
			// The check on self.active.contains()
			// here is probably unnecessary, since a
			// duplicate insert into next would be
			// harmless and cells with 3 active
			// neighbours already got inserted into
			// next in the loop above.
			(**num_neighbours == 3) && (!self.active.contains(pos)))
		.map(|(pos, _)| pos)) {
	    result.insert(*p)	// updates ranges also.
	}
	result
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

fn part_n(part_num: i32, initial: &Lattice, num_cycles: usize, use_w: bool) {
    let mut begin: Lattice = initial.clone();
    begin.use_w = use_w;
    let mut current: Lattice = begin.iterate();
    for _iteration in 1..num_cycles {
	current = current.iterate();
    }
    println!("Part {}: after {} iterations, population is {}",
	     part_num, num_cycles, current.popcount());
}


fn run() -> Result<(), String> {
    let initial = read_input()?;
    println!("Initial state is:\n{}", initial);
    part_n(1, &initial, 6, false);
    part_n(2, &initial, 6, true);
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
