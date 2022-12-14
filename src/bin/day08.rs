use std::fmt;
use std::io;
use std::io::BufRead;

#[derive(PartialEq, Copy, Clone)]
enum Operation {
    Nop,
    Jmp,
    Acc,
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            Operation::Nop => "nop",
            Operation::Jmp => "jmp",
            Operation::Acc => "acc",
        })
    }
}

fn flip_op(op: &Operation) -> Operation {
    match op {
        Operation::Nop => Operation::Jmp,
        Operation::Jmp => Operation::Nop,
        Operation::Acc => Operation::Acc,
    }
}

#[derive(PartialEq, Copy, Clone)]
struct Instruction {
    op: Operation,
    arg: i32,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:<3} {:+}", self.op, self.arg)
    }
}

fn flip_instr(instr: &Instruction) -> Instruction {
    Instruction {
        op: flip_op(&instr.op),
        arg: instr.arg,
    }
}

struct CodeChange {
    location: usize,
    instruction: Instruction,
}

struct GameConsole {
    code: Vec<Instruction>,
    visits: Vec<u32>,
    accumulator: i32,
    program_counter: usize,
}

impl GameConsole {
    fn reset(&mut self, pc: usize) {
        self.visits.clear();
        self.visits.resize(self.code.len(), 0);
        self.accumulator = 0;
        self.program_counter = pc;
    }

    fn codesize(&self) -> usize {
        self.code.len()
    }

    fn disassemble(&mut self, start: usize, n: usize) {
        for (offset, instr) in self.code[start..].iter().take(n).enumerate() {
            println!("{:>4} {}", (start + offset), instr);
        }
    }

    fn get_instr(&self, loc: usize, change: &CodeChange) -> Instruction {
        if loc == change.location {
            change.instruction
        } else {
            self.code[loc]
        }
    }

    fn run_with_change(&mut self, pc: usize, verbose: bool, change: &CodeChange) -> (bool, usize) {
        self.reset(pc);
        loop {
            if verbose {
                print!(
                    "PC={:>4} ACC={:>+4}, executing instruction {}: ",
                    self.program_counter, self.accumulator, self.code[self.program_counter]
                );
            }
            self.visits[self.program_counter] = 1;
            let instr = self.get_instr(self.program_counter, change);
            let newpc = match instr.op {
                Operation::Nop => self.program_counter + 1,
                Operation::Jmp => {
                    let offset = instr.arg;
                    if offset > 0 {
                        self.program_counter + (offset as usize)
                    } else {
                        self.program_counter - ((-offset) as usize)
                    }
                }
                Operation::Acc => {
                    self.accumulator += instr.arg;
                    if verbose {
                        print!("new ACC={:>+4} ", self.accumulator);
                    }
                    self.program_counter + 1
                }
            };
            if verbose {
                println!("new PC={:>4}", newpc);
            }
            self.program_counter = newpc;
            if self.program_counter >= self.code.len() {
                return (true, self.program_counter);
            }
            if self.visits[self.program_counter] != 0 {
                return (false, self.program_counter);
            }
        }
    }

    fn run(&mut self, pc: usize, verbose: bool) -> (bool, usize) {
        let inaccessible_change = CodeChange {
            location: self.code.len(), // cannot be reached
            instruction: Instruction {
                op: Operation::Nop,
                arg: 0,
            },
        };
        self.run_with_change(pc, verbose, &inaccessible_change)
    }
}

fn decode_instruction(line: &str) -> Result<Instruction, String> {
    let mut fields = line.split_whitespace();
    let opcode = match fields.next() {
        None => {
            return Err("missing opcode".to_string());
        }
        Some(opcode_str) => match opcode_str {
            "nop" => Operation::Nop,
            "jmp" => Operation::Jmp,
            "acc" => Operation::Acc,
            _ => {
                return Err(format!("unknown opcode {}", opcode_str));
            }
        },
    };
    let arg: i32 = match fields.next() {
        None => {
            return Err("missing argument".to_string());
        }
        Some(arg_str) => match arg_str.parse() {
            Ok(arg) => arg,
            Err(e) => {
                return Err(format!("Integer parsing error: {}", e));
            }
        },
    };
    match fields.next() {
        None => Ok(Instruction { op: opcode, arg }),
        _ => Err(format!("spurious extra field in '{}'", line)),
    }
}

fn read_program() -> Result<GameConsole, String> {
    let mut console = GameConsole {
        code: Vec::new(),
        visits: Vec::new(),
        accumulator: 0,
        program_counter: 0,
    };
    for thing in io::BufReader::new(io::stdin()).lines() {
        match thing {
            Err(e) => return Err(format!("I/O error: {}", e)),
            Ok(line) => match decode_instruction(&line) {
                Ok(instr) => console.code.push(instr),
                Err(e) => {
                    return Err(format!("failed to decode instruction '{}': {}", line, e));
                }
            },
        }
    }
    Ok(console)
}

fn part1(console: &mut GameConsole) {
    let (terminated, pc) = console.run(0, false);
    println!(
        "Part 1: code {} at PC={} with ACC={:>+4}",
        (match terminated {
            true => "terminated normally",
            false => "entered an infinite loop",
        }),
        pc,
        console.accumulator
    );
}

fn part2(console: &mut GameConsole) {
    let changes: Vec<CodeChange> = console
        .code
        .iter()
        .enumerate()
        .filter(|(_, instr)| instr.op != Operation::Acc)
        .map(|(offset, instr)| CodeChange {
            location: offset,
            instruction: flip_instr(instr),
        })
        .collect();
    for change in &changes {
        let (terminated, pc) = console.run_with_change(0, false, change);
        if terminated {
            println!(
                "Part 2: console code with flip at {} terminated normally at PC={} with ACC={:>+4}",
                change.location, pc, console.accumulator
            );
            println!(
                "Part 2: code before flip: {}",
                console.code[change.location]
            );
            println!("Part 2: code  after flip: {}", change.instruction);
            return;
        }
    }
    println!("Part 2: failed to find a suitable flip");
}

fn run() -> Result<(), String> {
    let mut console = read_program()?;
    console.disassemble(0, console.codesize());
    part1(&mut console);
    part2(&mut console);
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
