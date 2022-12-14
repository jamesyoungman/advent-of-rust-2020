use std::io;
use std::io::BufRead;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Token {
    Number(i64),
    LeftParen,
    RightParen,
    Operator(char),
}

struct Lexer {
    data: Vec<char>,
    pos: usize,
    current: Option<Token>,
}

impl Lexer {
    fn new(s: &str) -> Lexer {
        let mut result = Lexer {
            data: s.chars().collect(),
            pos: 0,
            current: None,
        };
        result.consume();
        result
    }

    fn unread_input(&self) -> String {
        (self.pos..self.data.len()).map(|i| self.data[i]).collect()
    }

    fn next(&self) -> Option<Token> {
        self.current
    }

    fn consume(&mut self) {
        // replace current ...
        self.current = {
            // ... after skipping any spaces ...
            while self.pos != self.data.len() {
                if self.data[self.pos] != ' ' {
                    break;
                }
                self.pos += 1;
            }
            // ... and bailing out if we already reached end-of-input ...
            if self.pos == self.data.len() {
                None
            } else {
                // ... with the next token
                let ch = self.data[self.pos];
                match ch {
                    '(' => {
                        self.pos += 1;
                        Some(Token::LeftParen)
                    }
                    ')' => {
                        self.pos += 1;
                        Some(Token::RightParen)
                    }
                    '+' | '*' => {
                        self.pos += 1;
                        Some(Token::Operator(ch))
                    }
                    '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                        let mut result = String::new();
                        while self.pos < self.data.len() {
                            match self.data[self.pos] {
                                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                                    result.push(self.data[self.pos]);
                                    self.pos += 1
                                }
                                _ => {
                                    break;
                                }
                            }
                        }
                        Some(Token::Number(
                            result.parse().expect("surprisingly invalid integer"),
                        ))
                    }
                    _ => {
                        panic!("unexpected character '{}' in input", ch);
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
enum Expr {
    Constant(i64),                  // some integer
    Op(Box<Expr>, char, Box<Expr>), // some binary expression
}

#[derive(Clone, Copy)]
struct Rules {
    precedence_plus: i64,
    precedence_times: i64,
}

struct Parser {
    rules: Rules,
}

// A recursive descent parser for arithmetic expressions which uses
// configurable precedence values to avoid having separate
// implementations for parts 1 and 2.
//
// Based on http://www.engr.mun.ca/~theo/Misc/exp_parsing.htm
impl Parser {
    fn prec(&self, b: &Option<Token>) -> Result<i64, String> {
        match b {
            None => Ok(-1),
            Some(Token::Operator('+')) => Ok(self.rules.precedence_plus),
            Some(Token::Operator('*')) => Ok(self.rules.precedence_times),
            Some(Token::Operator(ch)) => Err(format!("[E0200] unknown operator {}", ch)),
            Some(t) => Err(format!("[E0300] unexpected token {:?}", t)),
        }
    }

    fn right_prec(&self, b: &Option<Token>) -> Result<i64, String> {
        self.prec(b).map(|p| if p < 0 { p } else { p + 1 })
    }

    fn next_prec(&self, b: &Option<Token>) -> Result<i64, String> {
        self.prec(b)
    }

    fn prec_is_between(&self, low: i64, t: &Option<Token>, high: i64) -> bool {
        match self.prec(t) {
            Ok(p) => {
                if p < low {
                    false
                } else if p > high {
                    false
                } else {
                    true
                }
            }
            _ => false,
        }
    }

    fn parse_paren_expr(&self, lex: &mut Lexer) -> Result<Expr, String> {
        lex.consume(); // consume the (
        self.parse_expression(0, lex) // and the contents
            .and_then(|r| {
                match lex.next() {
                    Some(Token::RightParen) => {
                        // and finally the closing )
                        lex.consume();
                        Ok(r)
                    }
                    _ => Err("expected a closing ')'".to_string()),
                }
            })
    }

    fn parse_operand(&self, lex: &mut Lexer) -> Result<Expr, String> {
        match lex.next() {
            Some(Token::LeftParen) => self.parse_paren_expr(lex),
            Some(Token::Number(n)) => {
                lex.consume();
                Ok(Expr::Constant(n))
            }
            _ => Err("[E0600] expected a number or an open parenthesis".to_string()),
        }
    }

    fn parse_expression(&self, precedence: i64, lex: &mut Lexer) -> Result<Expr, String> {
        assert!(precedence >= 0);
        let mut lhs: Expr = self.parse_operand(lex)?;
        let mut stop_at_prec: i64 = 1000;
        while self.prec_is_between(precedence, &lex.next(), stop_at_prec) {
            let operator = lex.next();
            lex.consume();
            lhs = match operator {
                Some(Token::Operator(ch)) if (ch == '+' || ch == '*') => {
                    let prec = self.right_prec(&operator)?;
                    let rhs = self.parse_expression(prec, lex)?;
                    Expr::Op(Box::new(lhs), ch, Box::new(rhs))
                }
                _ => {
                    return Err(format!("[E1050] expected '+' or '*', got '{:?}'", operator));
                }
            };
            stop_at_prec = self.next_prec(&operator)?;
        }
        Ok(lhs)
    }

    fn parse(&self, expr_str: &str) -> Result<Expr, String> {
        let mut lex = Lexer::new(expr_str);
        let tree = self.parse_expression(0, &mut lex)?;
        // check we parsed the whole expression
        match lex.next() {
            None => Ok(tree),
            Some(token) => Err(format!(
                "[E1100] unexpected {:?}; unread input is '{}'",
                token,
                lex.unread_input()
            )),
        }
    }
}

fn eval(tree: &Expr) -> i64 {
    match tree {
        Expr::Constant(n) => *n,
        Expr::Op(e1, op, e2) => {
            let v1 = eval(e1);
            let v2 = eval(e2);
            match op {
                '+' => v1 + v2,
                '*' => v1 * v2,
                _ => {
                    panic!("unexpected operator '{}'", op);
                }
            }
        }
    }
}

fn parse_evaluate_and_total(
    part_number: i32,
    input: &Vec<String>,
    show_calcs: bool,
    rules: &Rules,
) -> Result<(), String> {
    let p = Parser {
        rules: rules.clone(),
    };

    let mut total: i64 = 0;
    for line in input {
        let tree = p.parse(&line)?;
        let value = eval(&tree);
        if show_calcs {
            println!("{} -> {}", line, value);
        }
        total += value;
    }
    println!("Part {}: total = {:?}", part_number, total);
    Ok(())
}

fn part1(input: &Vec<String>, show_calcs: bool) -> Result<(), String> {
    parse_evaluate_and_total(
        1,
        input,
        show_calcs,
        &Rules {
            precedence_plus: 20,
            precedence_times: 20,
        },
    )
}

fn part2(input: &Vec<String>, show_calcs: bool) -> Result<(), String> {
    parse_evaluate_and_total(
        2,
        input,
        show_calcs,
        &Rules {
            precedence_plus: 30,
            precedence_times: 20,
        },
    )
}

fn read_input() -> Result<Vec<String>, String> {
    let mut input_lines: Vec<String> = Vec::new();
    for input_item in io::BufReader::new(io::stdin()).lines() {
        match input_item {
            Err(e) => {
                return Err(format!("I/O error: {}", e));
            }
            Ok(item) => {
                input_lines.push(item);
            }
        }
    }
    Ok(input_lines)
}

fn run() -> Result<(), String> {
    let input: Vec<String> = read_input()?;
    let show_calcs = false;
    part1(&input, show_calcs)?;
    part2(&input, show_calcs)?;
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
