use std::io::BufRead;
use std::string::String;
use std::io;


fn ids_with_positions(bus_ids: &Vec<String>) -> Result<Vec<(i64, i64)>, String> {
    let mut result = Vec::new();
    for (i, bus) in bus_ids.iter().enumerate() {
	if bus.as_str() != "x" {
	    let n: i64 = match bus.parse() {
		Err(e) => {
		    return Err(format!("unable to parse '{}' as an integer: {}",
				       bus, e));
		}
		Ok(n) => n,
	    };
	    result.push((i as i64, n));
	}
    }
    Ok(result)
}


fn solve1(earliest_departure: i64, bus_ids: &Vec<String>) -> Result<(i64, i64), String> {
    let buses: Vec<i64> = match ids_with_positions(bus_ids) {
	Err(e) => {
	    return Err(e);
	}
	Ok(things) => things.iter().map(|(_,b)| b).cloned().collect()
    };
    let mut first_bus: Option<i64> = None;
    let mut first_bus_departs_at: Option<i64> = None;
    for bus_id in buses {
        // In which cycle of this bus does the earliest departure time
        // fall?
        let cycle = earliest_departure / bus_id;
        let next = if earliest_departure % bus_id != 0 {
	    (cycle + 1) * bus_id
	} else {
            earliest_departure
	};
        if first_bus_departs_at.is_none() || next < first_bus_departs_at.unwrap() {
            first_bus = Some(bus_id);
            first_bus_departs_at = Some(next)
	}
    }
    match (first_bus, first_bus_departs_at) {
	(Some(a), Some(b)) => Ok((a, b)),
	_ => Err("failed to solve part 1".to_string()),
    }
}

fn part1(earliest: &i64, bus_ids: &Vec<String>) -> Result<(), String> {
    let (first_bus, departure) = match solve1(*earliest, bus_ids) {
	Err(e) => {
	    return Err(e);
	}
	Ok((a, b)) => (a, b)
    };
    let wait = departure - earliest;
    println!("Part 1: we depart on bus {} in {} minutes; {}*{} = {}",
	     first_bus, wait, first_bus, wait, (first_bus * wait));
    Ok(())
}


fn read_input(reader: impl BufRead) -> Result<(i64, Vec<String>), String> {
    let mut it = reader.lines();
    let mut getline = || {
	match it.next() {
	    None => {
		return Err("premature end of file".to_string());
	    }
	    Some(Err(e)) => {
		return Err(format!("I/O error: {}", e));
	    }
	    Some(Ok(s)) => Ok(s),
	}
    };

    let earliest: i64 = match getline()?.parse() {
	Err(e) => {
	    return Err(format!("unable to parse integer from input: {}", e));
	}
	Ok(n) => n,
    };
    let result: Vec<String> = getline()?.split(",").map(|s| s.to_string()).collect();
    Ok((earliest, result))
}

fn run() -> Result<(), String> {
    let (earliest, bus_ids) = read_input(io::BufReader::new(io::stdin()))?;
    part1(&earliest, &bus_ids)?;
    //part2(&bus_ids)?;
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
