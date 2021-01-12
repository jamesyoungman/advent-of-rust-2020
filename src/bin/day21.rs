extern crate log;
extern crate pretty_env_logger;
#[macro_use] extern crate lazy_static;
extern crate regex;

use std::io;
use std::io::BufRead;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use regex::Regex;

type StringSet = HashSet<String>;

lazy_static! {
    static ref LINE_RE: Regex = Regex::new("^(.*) [(]contains ([^)]*)[)]$").unwrap();
}

fn parse(lines: &Vec<String>) -> Result<Vec<(StringSet, StringSet)>, String> {
    let mut result: Vec<(StringSet, StringSet)> = Vec::new();
    for line in lines {
	match LINE_RE.captures(line) {
	    None => {
		return Err("invalid input line".to_string());
	    }
	    Some(cap) => {
		result.push((cap[1].split(" ").map(str::to_owned).collect(),
			     cap[2].split(", ").map(str::to_owned).collect()));
	    }
	}
    }
    Ok(result)
}


#[derive(Debug)]
struct Ingredient {
    name: String,
    possible_allergens: StringSet,
    excluded_allergens: StringSet,
    definite_allergen: Option<String>,
}

impl Ingredient {
    fn new(name: &str, all_allergens: &StringSet) -> Ingredient {
	Ingredient{
	    name: name.to_string(),
	    possible_allergens: all_allergens.clone(),
	    excluded_allergens: StringSet::new(),
	    definite_allergen: None,
	}
    }

    fn is_non_allergenic(&self) -> Option<bool> {
	if self.definite_allergen.is_some() {
	    Some(false)
	} else if self.possible_allergens.len() > 0 {
	    None
	} else  {
	    Some(true)
	}
    }

    // If the set contains only one item, return it.  Otherwise return
    // None.
    fn sole_possible_allergen(&self) -> Option<String> {
	let mut it = self.possible_allergens.iter();
	match it.next() {
	    None => None,	// no items in the set
	    Some(candidate) => { // we found an item
		match it.next() {
		    None => Some(candidate.to_string()), // it was the only one
		    _ => None,	// there were others
		}
	    }
	}
    }

    fn conclude_must_contain(&mut self, allergen: &str) {
	assert!(!self.excluded_allergens.contains(allergen));
	assert!(self.definite_allergen.is_none());
	self.definite_allergen = Some(allergen.to_string());
	self.possible_allergens.remove(allergen);
	assert!(self.possible_allergens.is_empty());
    }

    fn conclude_must_not_contain(&mut self, allergen: &str) {
	match &self.definite_allergen {
	    None => (),
	    Some(a) => {
		assert!(a != allergen, "we already thought this ingredient did contain that");
	    }
	}
	self.excluded_allergens.insert(allergen.to_string());
	self.possible_allergens.remove(allergen);
    }
}


fn count_unknowns(all_ingredients: &HashMap<String, Ingredient>) -> usize {
    all_ingredients.values()
	.map(|ing| ing.possible_allergens.len())
	.sum()
}

fn deduce_sole_possible_allergens(all_ingredients: &HashMap<String, Ingredient>)
				  -> Vec<(String, String)> {
    let mut discoveries: Vec<(String, String)> = Vec::new();
    for (ing_name, ing) in all_ingredients {
	match ing.sole_possible_allergen() {
	    None => (),
	    Some(candidate) => {
		discoveries.push((ing_name.to_string(), candidate.to_string()));
	    }
	}
    }
    discoveries
}


fn solve1(lines: &Vec<String>)
	  -> Result<(Vec<(StringSet, StringSet)>,
		     HashMap<String, Ingredient>),
		    String> {
    let parsed_input: Vec<(StringSet, StringSet)> = parse(lines)?;
    let all_allergens: StringSet =
	parsed_input.iter().map(|(_, a)| a).flatten().cloned().collect();
    let mut all_ingredients: HashMap<String, Ingredient> =
	parsed_input.iter().map(|(i, _)| i).flatten()
	.map(|ing_name| (ing_name.to_string(), Ingredient::new(ing_name, &all_allergens)))
	.collect();
    let all_ingredient_names: StringSet = all_ingredients.keys().cloned().collect();

    for (ingredients, allergens) in &parsed_input {
	// One of these ingredients listed on this line contains each
	// of these allergens.  Therefore any ingredient not listed
	// here cannot contain any of these allergens, since we are
	// told that any allergen occurs in just one ingredient.
	for allergen in allergens {
	    for ing_name in all_ingredient_names.difference(ingredients) {
		all_ingredients.get_mut(ing_name).unwrap().conclude_must_not_contain(allergen);
	    }
	}
    }

    loop {
	let prev_unknowns = count_unknowns(&all_ingredients);
	if prev_unknowns == 0 {
	    log::info!("part1: solved");
	    break;
	}
	log::debug!("all_ingredients: {:?}", all_ingredients);
	let discoveries = deduce_sole_possible_allergens(&all_ingredients);

	for (has_it, allergen) in discoveries {
	    for (ing_name, ing) in all_ingredients.iter_mut() {
		if *ing_name == has_it {
		    ing.conclude_must_contain(&allergen);
		} else {
		    ing.conclude_must_not_contain(&allergen);
		}
	    }
	}
	let curr_unknowns = count_unknowns(&all_ingredients);
	if curr_unknowns == prev_unknowns {
	    return Err("not solvable".to_string());
	}
    }

    let allergenic_ingredients: HashMap<String, String> = all_ingredients.values()
		  .filter_map(|ing| match ing.definite_allergen.as_ref() {
		      Some(a) => Some((ing.name.to_string(), a.to_string())),
		      None => None
		  })
		  .collect();
    let width: usize = allergenic_ingredients.iter().map(|(n, _)| n.len()).max().unwrap_or(1);
    for (ing_name, allergen) in &allergenic_ingredients {
	println!("{:<width$} contains {}", ing_name, allergen, width=width);
    }
    Ok((parsed_input, all_ingredients))
}

fn part1(lines: &Vec<String>) -> Result<HashMap<String, Ingredient>, String> {
    let (parsed_input, ingredients) = solve1(lines)?;
    let non_alllergenic: StringSet = ingredients.values()
	.filter_map(|ing| match ing.is_non_allergenic() {
	    Some(true) => Some(ing.name.to_string()),
	    _ => None,
	})
	.collect();
    let mentions = parsed_input.iter()
	.flat_map(|(ingredent_names, _)| ingredent_names.iter().cloned())
	.filter(|ing_name| non_alllergenic.contains(ing_name))
	.count();

    println!("Part 1: non-allergen count is {}", mentions);
    Ok(ingredients)
}


fn part2(all_ingredients: &HashMap<String, Ingredient>) -> Result<(), String> {
    // We use a BTreeMap here specifically because we want to rely on
    // iteration in key order.
    let allergen_to_ingredient_map: BTreeMap<String, String> =
	all_ingredients.values()
	.filter_map(|ing| match &ing.definite_allergen {
	    Some(a) => Some((a.to_string(), ing.name.to_string())),
	    None => None,
	})
	.collect();
    println!("Part 2: ingredients sorted by allergen name: {}",
	     itertools::join(allergen_to_ingredient_map.iter()
			     .map(|(_, ingredient_name)| ingredient_name),
			     ","));
    Ok(())
}

fn run() -> Result<(), String> {
    let mut lines: Vec<String> = Vec::new();
    for line_or_err in io::BufReader::new(io::stdin()).lines() {
	match line_or_err {
	    Err(e) => {
		return Err(format!("I/O error: {}", e));
	    }
	    Ok(line) => {
		lines.push(line);
	    }
	}
    }
    let ingredients = part1(&lines)?;
    part2(&ingredients)?;
    Ok(())
}

fn main() {
    // the env logger is configured with $RUST_LOG.
    // For example RUST_LOG=debug day20
    pretty_env_logger::init();
    std::process::exit(match run() {
	Ok(_) => 0,
	Err(err) => {
	    eprintln!("error: {}", err);
	    1
	}
    });
}
