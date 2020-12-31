use std::io;
use std::collections::BTreeMap;
use std::io::BufRead;


fn differences(ratings: &Vec<i64>) -> Vec<(i64, i64)> {
    let mut result: Vec<(i64, i64)> = Vec::new();
    result.reserve(ratings.len());
    let mut last = 0;
    for rating in ratings {
	result.push((rating-last, *rating));
	last = *rating
    }
    result.push((3, last+3));
    result
}


fn read_i64(thing: Result<String, std::io::Error>) -> Result<i64, String> {
    match thing {
	Err(e) => Err(format!("I/O error: {}", e)),
	Ok(line) => match line.parse::<i64>() {
	    Err(e) => Err(format!("unable to parse '{}' as an integer: {}", line, e)),
	    Ok(n) => Ok(n),
	}
    }
}

fn sorted_integer_input() -> Result<Vec<i64>, String> {
    let mut items: Vec<i64> = match io::BufReader::new(io::stdin())
	.lines().map(read_i64).collect() {
	    Err(e) => return Err(e),
	    Ok(numbers) => numbers,
	};
    items.sort();
    Ok(items)
}

fn part1(ratings: &Vec<i64>) -> (Vec<(i64, i64)>, i64) {
    let diffs = differences(ratings);
    let my_device_rating: i64 = (*diffs.last().unwrap()).1;
    println!("Part 1: my device rating is {}", my_device_rating);
    let mut counts: BTreeMap<i64, usize> = BTreeMap::new();
    for (d, _) in &diffs {
	match d {
	    1 | 2 | 3 => {
		*counts.entry(*d).or_insert(0) += 1;
	    }
	    _ => {
		panic!(format!("unexpected diff {}", d));
	    }
	}
    }
    let solution: usize = counts.get(&1).unwrap_or(&0) * counts.get(&3).unwrap_or(&0);
    println!("Part 1: answer us {}", solution);
    (diffs, my_device_rating)
}

fn run() -> Result<(), String> {
    let ratings = sorted_integer_input()?;
    let (diffs, my_device_rating) = part1(&ratings);
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
