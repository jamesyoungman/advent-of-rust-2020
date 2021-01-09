#[macro_use]
extern crate lazy_static;
extern crate ndarray;
extern crate regex;

use ndarray::prelude::*;
use ndarray::s;
use regex::Regex;
use std::collections::HashMap;
use std::fmt;
use std::io;
use std::io::Read;
use std::str::FromStr;
use std::num::ParseIntError;

lazy_static! {
    static ref TILE_TITLE_RE: Regex = Regex::new("^Tile ([0-9]*):$").unwrap();
}

#[derive(Debug,Copy,Clone)]
struct TileId {
    val: i32
}

impl From<i32> for TileId {
    fn from(n: i32) -> Self {
	TileId{ val: n }
    }
}

impl FromStr for TileId {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
	match s.parse() {
	    Err(e) => Err(e),
	    Ok(n) => Ok(TileId{val: n}),
	}
    }
}

impl fmt::Display for TileId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
	write!(f, "{}", self.val)
    }
}

#[derive(Debug,Copy,Clone)]
enum Rotation {
    Zero,
    One,
    Two,
    Three
}

#[derive(Copy,Clone)]
struct Manipulation {
    rot: Rotation,
    flip: bool,
}

impl Manipulation {
    fn new(n: i32) -> Manipulation {
	Manipulation {
	    rot: match n & 0x03 {
		0 => Rotation::Zero,
		1 => Rotation::One,
		2 => Rotation::Two,
		3 => Rotation::Three,
		_ => panic!("implossible"),
	    },
	    flip: match n & 0x04 {
		0 => false,
		4 => true,
		_ => panic!("implossible"),
	    }
	}
    }

    fn all() -> Vec<Manipulation> {
	(0..8).map(Manipulation::new).collect()
    }

    fn as_string(&self) -> String {
	format!("R{}F{}",
		match self.rot {
		    Rotation::Zero => 0,
		    Rotation::One => 1,
		    Rotation::Two => 2,
		    Rotation::Three => 3,
		},
		if self.flip { "Y" } else { "N" })
    }

    fn do_rot(&self, tiledata: Array2<u8>) -> Array2<u8> {
	match self.rot {
	    Rotation::Zero => tiledata,
            Rotation::One => tiledata.slice(s![.., ..;-1]).reversed_axes().into_owned(),
            Rotation::Two => tiledata.slice(s![..;-1, ..;-1]).into_owned(),
            Rotation::Three => tiledata.slice(s![..;-1, ..]).reversed_axes().into_owned(),
	}
    }

    fn do_flip(&self, tiledata: Array2<u8>) -> Array2<u8> {
	if self.flip {
	    tiledata.slice(s![.., ..;-1]).into_owned()
	} else {
	    tiledata
	}
    }

    fn on(&self, m: &Array2<u8>) -> Array2<u8> {
	self.do_rot(self.do_flip(m.to_owned()))
    }
}

impl FromStr for Manipulation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
	let mut it = s.chars();
	if let Some(ch) = it.next() {
	    if ch != 'R' {
		return Err("must begin with R".to_string());
	    }
	}
	let rot = match it.next() {
	    None => { return Err("string is too short".to_string()); },
	    Some('0') => Rotation::Zero,
	    Some('1') => Rotation::One,
	    Some('2') => Rotation::Two,
	    Some('3') => Rotation::Three,
	    Some(ch) => { return Err(format!("invalid rotation {}", ch)); },
	};
	if let Some(ch) = it.next() {
	    if ch != 'F' {
		return Err("must have F as the third character".to_string());
	    }
	}
	let flip = match it.next() {
	    Some('Y') => true,
	    Some('N') => false,
	    _ => { return Err("flip must be Y or N".to_string()); },
	};
	match it.next() {
	    None => Ok(Manipulation{rot, flip}),
	    _ => Err("trailing garbage at the end of the string".to_string()),
	}
    }
}


impl fmt::Display for Manipulation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
	f.write_str(&self.as_string())
    }
}

impl fmt::Debug for Manipulation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
	f.write_str(&self.as_string())
    }
}




#[derive(Debug)]
struct Tile {
    id: TileId,
    d: Array2<u8>,
}

fn decode_ascii_tile(r: usize,
		     c: usize,
		     width: &usize,
		     s: &Vec<char>) -> u8 {
    let pos: usize = (width+1) * r + c;
    match s[pos] {
	'#' => 1,
	'.' => 0,
	_ => 2,	// signal an error.
    }
}

impl Tile {
    fn from_string(s: &str) -> Result<Tile, String> {
	let lines: Vec<String> = s.split('\n').map(str::to_string).collect();
	if lines.len() == 0 {
	    return Err("Tiles must not be empty".to_string());
	}
	let id: TileId = match TILE_TITLE_RE.captures(&lines[0]) {
	    Some(caps) => match caps[1].parse() {
		Ok(n) => n,
		Err(e) => { return Err(format!("failed to parse '{}' as an integer: {}",
					       &caps[1], e)); }
	    }
	    None => { return Err("tile is missing a title".to_string()); }
	};
	println!("tile id is {}", id);
	let width = lines[1].len();
	let height = lines.len() - 1;
	if width != height {
	    return Err(format!("Tiles should be square but this has {} rows, {} columns: {:?}",
			       height, width, lines));
	}
	let tiledata = s[width+1..].chars().collect::<Vec<char>>();
	let d = Array::from_shape_fn(
	    (height, width), |(r, c)| decode_ascii_tile(r, c, &width, &tiledata));
	if d.iter().filter(|x| **x == 2).count() > 0 {
	    return Err("tile data contained unexpected characters".to_string());
	}
	Ok(Tile{
	    id,
	    d,
	})
    }
}



fn self_test() -> Result<(), String> {
    Ok(())
}

fn read_tiles(s: &str) -> Vec<Tile> {
    let r: Result<Vec<Tile>, String> = s.split("\n\n")
	.map(|s| s.trim() )
	.map(Tile::from_string)
	.collect();
    return r.expect(format!("tiles are not in the expected format").as_str());
}

fn matrix_key<T: std::string::ToString>(m: &Array2<T>) -> String {
    m.iter().map(|x| x.to_string()).collect()
}

fn matrix_edge(m: &Array2<u8>, edge: char) -> String {
    let slice = match edge {
	'N' => s![0, ..].clone(),
	'E' => s![.., m.ncols()-1].clone(),
	'S' => s![m.nrows()-1, ..].clone(),
	'W' => s![.., 0].clone(),
	_ => panic!(format!("matrix_edge called with incorrect edge name {}", edge)),
    };
    let pattern: String = m.slice(&slice).iter()
	.map(|x| match x {
	    0 => '.',
	    1 => '#',
	    _ => panic!("matrix should be 0/1 only"),
	})
	.collect();
    format!("{}{}", edge, pattern)
}

fn get_edges(m: &Array2<u8>) -> [String;4] {
    [
	matrix_edge(m, 'N'),
	matrix_edge(m, 'E'),
	matrix_edge(m, 'S'),
	matrix_edge(m, 'W')
    ]
}


fn make_tile_index(tiles: &Vec<Tile>) -> HashMap<String, (TileId, Manipulation)> {
    let mut result: HashMap<String, (TileId, Manipulation)> = HashMap::new();
    for t in tiles {
	let variants: Vec<(Manipulation, TileId, Array2<u8>)> = Manipulation::all()
	    .iter()
	    .map(|manip| (*manip, t.id, manip.on(&t.d)))
	    .collect();
	let mut seen_keys: HashMap<String, Vec<(TileId, Manipulation)>> = HashMap::new();
	for (manip, tid, v) in &variants {
	    let mk = matrix_key(v);
	    seen_keys.entry(mk.clone()).or_insert_with(Vec::new).push((*tid, *manip));
	    assert_eq!(seen_keys.get(&mk).unwrap().len(), 1);

	    for edge_key in get_edges(v).iter() {
		let e = (*tid, *manip);
		result.insert(edge_key.to_string(), e);
	    }
	}
    }
    result
}

fn part1(tiles: &Vec<Tile>) -> Result<(), String> {
    let ix = make_tile_index(tiles);
    println!("part1: tile index is: {:?}", ix);
    Ok(())
}

fn run() -> Result<(), String> {
    self_test()?;
    let mut buffer = String::new();
    let tiles = match io::stdin().read_to_string(&mut buffer) {
	Ok(_) => read_tiles(&buffer),
	Err(e) => { return Err(format!("I/O error: {}", e)); }
    };
    for t in &tiles {
	println!("{:?}", t);
    }
    part1(&tiles)?;
    Ok(())
}

fn main() {
    std::process::exit(match run() {
	Ok(_) => 0,
	Err(err) => {
	    eprintln!("error: {}", err);
	    1
	}
    });
}
