extern crate itertools;
use std::io::BufRead;
use std::string::String;
use std::vec::Vec;
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


fn modinv(u: i64, v: i64) -> Option<i64> {
    // Determines the multiplicative inverse of u modulo v, returning
    // a value >= 0 or None if no inverse exists.  This is based on
    // Knuth's Algorithm X for the Extended GCD of u and v
    // (Seminumerical Algorithms, section 4.5.2 "The Greatest Common
    // Divisor").
    //
    // The non-negativity of the results of this function are a key
    // feature of this solution.
    //
    // Since we're only trying to find one multiplicative inverse
    // and not the gcd, we don't require Knuth's u2 or v2.
    let (mut u1, mut u3): (i64, i64) = (1, u);
    let (mut v1, mut v3): (i64, i64) = (0, v);
    let mut iter = 1;		// sign of iter flips each iteration.
    while v3 != 0 {
	// Step X3.
	let q = u3 / v3;
	let t3 = u3 % v3;
	let t1 = u1 + v1 * q;
	u1 = v1;
	v1 = t1;
	u3 = v3;
	v3 = t3;
	iter = -iter;
    }
    match u3 {
	1 => if iter < 0 {
	    Some(v - u1)
	} else {
	    Some(u1)
	}
	_ => None,
    }
}


// Find a congruent value t where for all i,
//  t == residue[i] mod modulus[i]
// Returns (t, M) where M is the product of the moduli.
fn crt(residues_and_moduli: &[(i64, i64)]) -> (i64, i64) {
    // If any modulus is negative we would need need to change the
    // code to exercise more care in use of %.
    assert!(residues_and_moduli.iter().all(|rm| rm.1 > 0));
    let p = residues_and_moduli.iter().map(|rm| rm.1).product();
    let mut v = 0;
    for (u, m) in residues_and_moduli.iter() {
	let e = p / m;
	// The use of % to compute moduli here depends on the fact
	// that our modinv implementation never returns s < 0.
	let s = modinv(e, *m).expect("e has no multiplicative inverse mod m");
	assert!(s >= 0);
	v += e * (u * (s % *m));
    }
    let result = v % p;
    (result, p)
}


fn solve2(bus_ids: &Vec<String>) -> Result<i64, String> {
    let buses: Vec<(i64, i64)> = ids_with_positions(bus_ids)?;
    let (n, mm): (i64, i64) = crt(&buses);
    assert!(n >= 0);
    if mm - n > 0 {
	Ok(mm - n)
    } else {
	Ok(n)
    }
}

fn part2(bus_ids: &Vec<String>) -> Result<(), String> {
    println!("Part 2: result is {}", solve2(bus_ids)?);
    Ok(())
}


fn self_test() -> Result<(), String> {
    let cases: &[(&str, &str, i64)] = &[
	("example-0", "7,13,x,x,59,x,31,19", 1068781),
	("example-1", "17,x,13,19", 3417),
	("example-2", "67,7,59,61", 754018),
	("example-3", "67,x,7,59,61", 779210),
	("example-4", "67,7,x,59,61", 1261476),
	("example-5", "1789,37,47,1889", 1202161486),
    ];
    fn run_test_case(label: &str, input: &str, expected: i64) -> Result<(), String> {
	let id_list = input.split(",").map(|s| s.to_string()).collect();
	let got: i64 = solve2(&id_list)?;
	if got != expected {
	    return Err(format!("FAIL: {}: for input {}, expected {} but got {}",
			       label, input, expected, got));
	}
	Ok(())
    }
    let mut failures: Vec<String> = Vec::new();
    for t in cases {
	match run_test_case(t.0, t.1, t.2) {
	    Err(e) => {
		eprintln!("FAIL: {}", e);
		failures.push(e);
		break;
	    }
	    Ok(_) => {
		println!("PASS: {}", t.0);
	    }
	};
    }
    match failures.iter().next() {
	None => Ok(()),
	Some(msg) => Err(msg.to_string()), // just the first failure.
    }
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
    self_test()?;
    let (earliest, bus_ids) = read_input(io::BufReader::new(io::stdin()))?;
    part1(&earliest, &bus_ids)?;
    part2(&bus_ids)?;
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
