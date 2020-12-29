use std::collections::HashSet;
use std::io;
use std::io::BufRead;

#[derive(Hash, Eq, PartialEq, Debug)]
struct Pos {
    x: usize,
    y: usize,
}

fn count_trees(tree_positions: &HashSet<Pos>,
	       dx: usize, dy: usize,
	       map_width: usize,
	       slope_height: usize) -> usize {
    let mut x: usize = 0;
    let mut y: usize = 0;
    let mut count: usize = 0;
    while y < slope_height {
	if tree_positions.contains(&Pos{x: x, y: y}) {
	    count += 1;
	}
	x = (x + dx) % map_width;
	y = y + dy;
    }
    count
}

fn part1(tree_positions: &HashSet<Pos>,
	 map_width: usize,
	 slope_height: usize) -> usize {
    return count_trees(tree_positions, 3, 1, map_width, slope_height);
}

fn part2(tree_positions: &HashSet<Pos>,
	 map_width: usize,
	 slope_height: usize) -> usize {
    let mut product: usize = 1;
    let deltas = &[Pos{x: 1, y: 1},
		   Pos{x: 3, y: 1},
		   Pos{x: 5, y: 1},
		   Pos{x: 7, y: 1},
		   Pos{x: 1, y: 2}];
    for d in deltas.iter() {
	let count = count_trees(tree_positions, d.x, d.y,
				map_width, slope_height);
	product *= count;
    }
    product
}

fn read_line(line: &str, y: usize,
	     tree_positions: &mut HashSet<Pos>) -> usize {
    let mut width = 0;
    for (x, c) in line.chars().enumerate() {
	width = x + 1;
	if c == '#' {
	    tree_positions.insert(Pos{x: x,y: y});
	}
    }
    width
}

fn read_map(reader: impl BufRead) -> Result<
	(HashSet<Pos>, usize, usize),
    io::Error> {
    let mut tree_positions = HashSet::new();
    let mut map_width: usize = 0;
    let mut slope_height: usize = 0;
    for (y, line_or_fail) in reader.lines().enumerate() {
	match line_or_fail {
	    Ok(line) => {
		let w = read_line(&line, y, &mut tree_positions);
		if w > map_width {
		    map_width = w;
		}
		slope_height = y + 1;
	    }
	    Err(e) => {
		return Err(e);
	    }
	}
    }
    return Ok((tree_positions, map_width, slope_height));
}


fn run() -> Result<(), std::io::Error> {
    let reader = io::BufReader::new(io::stdin());
    let (tree_positions, map_width, slope_height) = read_map(reader)?;
    println!("Part 1: encountered {} trees",
	     part1(&tree_positions, map_width, slope_height));
    println!("Part 2: product is {}",
	     part2(&tree_positions, map_width, slope_height));
    return Ok(());
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
