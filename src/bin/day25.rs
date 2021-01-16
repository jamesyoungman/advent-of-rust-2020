extern crate log;
extern crate mod_exp;
extern crate pretty_env_logger;
extern crate thiserror;

use mod_exp::mod_exp;
use std::convert::TryInto;
use std::io::Read;
use std::io;
use thiserror::Error;

type Key = i64;


#[derive(Error,Debug)]
pub enum MyError {
    #[error("input is invalid; {0}")]
    InvalidInput(String),

    #[error("Read error")]
    ReadError { source: std::io::Error },

    #[error("Key {0} should not be negative: {1}")]
    NegativeKey(i64, std::num::TryFromIntError),

    #[error("numeric overflow")]
    Overflow,
}

fn read_input() -> Result<Vec<Key>, MyError> {
    let mut buffer = String::new();
    match io::stdin().read_to_string(&mut buffer) {
	Ok(_) => (),
	Err(source) => { return Err(MyError::ReadError{source}); },
    };
    let mut public_keys: Vec<Key> = Vec::new();
    for item in buffer.split("\n").filter(|s| !s.is_empty()) {
	match item.parse() {
	    Err(e) => { return Err(MyError::InvalidInput(format!("{}", e))) },
	    Ok(n) => { public_keys.push(n); }
	}
    }
    Ok(public_keys)
}

// Finds r such that 7^r = p mod 20201227
fn find_loop_num(p: Key) -> usize {
    let mut v = 1;
    for loop_number in 0.. {
	if v == p {
	    return loop_number;
	}
	v = (v * 7) % 20201227;
    }
    panic!("infinite loop terminated");
}

// Finds p = k^r mod 20201227
fn make_private_key(k: Key, r: usize) -> Result<i64, MyError> {
    match k.try_into() {
	Ok(ku) => mod_exp::<usize>(ku, r, 20201227)
	    .try_into().map_err(|_| MyError::Overflow),
	Err(e) => { return Err(MyError::NegativeKey(k, e)) }
    }
}


fn run() -> Result<(), MyError> {
    let public_keys = read_input()?;
    let keys_and_loop_numbers: Vec<(Key, usize)> = public_keys.iter()
	.map(|k| (*k, find_loop_num(*k))).collect();
    for (pk, loop_num) in &keys_and_loop_numbers {
	println!("public key is  {:>8}", pk);
	println!("loop number is {:>8}", loop_num);
	println!("");
    }
    let ek0 = make_private_key(keys_and_loop_numbers[0].0, keys_and_loop_numbers[1].1)?;
    println!("encryption key is {:>8}", ek0);
    let ek1 = make_private_key(keys_and_loop_numbers[1].0, keys_and_loop_numbers[0].1)?;
    assert_eq!(ek0, ek1);
    Ok(())
}

fn main() {
    // the env logger is configured with $RUST_LOG.
    // For example RUST_LOG=debug day24
    pretty_env_logger::init();
    std::process::exit(match run() {
	Ok(_) => 0,
	Err(err) => {
	    eprintln!("error: {}", err);
	    1
	}
    });
}
