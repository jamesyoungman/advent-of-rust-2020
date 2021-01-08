#[macro_use]
extern crate lazy_static;
extern crate ndarray;
extern crate regex;

use ndarray::prelude::*;
use regex::Regex;
use std::io;
use std::io::Read;

lazy_static! {
    static ref TILE_TITLE_RE: Regex = Regex::new("^Tile ([0-9]*):$").unwrap();
}

#[derive(Debug)]
struct Tile {
    id: i32,
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
	let id: i32 = match TILE_TITLE_RE.captures(&lines[0]) {
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
