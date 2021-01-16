extern crate log;
extern crate pretty_env_logger;
extern crate thiserror;

use std::io::Read;
use std::io;
use thiserror::Error;

type Key = i64;
static MODULUS: i64 = 20201227;

#[derive(Error,Debug)]
pub enum MyError {
    #[error("input is invalid; {0}")]
    InvalidInput(String),
    #[error("Read error")]
    ReadError { source: std::io::Error },
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

fn find_loop_num(subject: Key, public_key: Key) -> usize {
    let mut v = 1;
    for loop_number in 0.. {
	if v == public_key {
	    return loop_number;
	}
	v = (v * subject) % MODULUS;
    }
    panic!("infinite loop terminated");
}

fn make_private_key(subject: Key, loop_num: usize) -> Key {
    let mut v = 1;
    for _ in 0..loop_num {
	v = (v * subject) % MODULUS;
    }
    v
}


fn run() -> Result<(), MyError> {
    let public_keys = read_input()?;
    let keys_and_loop_numbers: Vec<(Key, usize)> = public_keys.iter()
	.map(|k| (*k, find_loop_num(7, *k))).collect();
    for (pk, loop_num) in &keys_and_loop_numbers {
	println!("public key is  {:>8}", pk);
	println!("loop number is {:>8}", loop_num);
	println!("");
    }
    let ek0 = make_private_key(keys_and_loop_numbers[0].0, keys_and_loop_numbers[1].1);
    let ek1 = make_private_key(keys_and_loop_numbers[1].0, keys_and_loop_numbers[0].1);
    println!("encryption key is {:>8}", ek0);
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
