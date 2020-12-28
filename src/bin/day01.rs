use std::collections::HashSet;
use std::io;
use std::io::prelude::*;


fn findpair(h: &HashSet<u64>, total: &u64, exclude: &u64) -> Result<(u64, u64), &'static str> {
   for x in h {
       if x != exclude && x < total {
              let y = total - x;
	      // We assume each number occurs only once, hence the use of y != *x
       	      // prevents us returning (total/2, total/2).
       	      if y != *exclude && y != *x && h.contains(&y) {
       	      	 return Ok((*x, y));
	      }
       }
   }
   return Err("did not find pair")
}

fn findtriple(h: &HashSet<u64>, total: &u64) -> Result<(u64, u64, u64), &'static str> {
   for c in h {
       let x = total - c;
       // Exclude c from consideraition, as we already have in our candidate triple.
       if let Ok((a, b)) = findpair(&h, &x, &c) {
       	     return Ok((a, b, *c));
       }
   }
   return Err("did not find triple")
}

fn main() {
    let h: HashSet<u64> =
        io::BufReader::new(io::stdin()).lines()
    	   .map(|s| s.unwrap().parse::<u64>().unwrap())
	          .collect();
    let (a, b) = findpair(&h, &2020, &0).unwrap();
    println!("Part 1: {}*{} = {}", a, b, a*b);
    let (a, b, c) = findtriple(&h, &2020).unwrap();
    println!("Part 2: {}*{}*{} = {}", a, b, c, a*b*c);
}
