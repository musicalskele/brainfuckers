use std::io::Read;

#[derive(Clone, Debug, PartialEq)]
enum OpCode {
    Move(i32),
    IncrementPointer, DecrementPointer,
    Increment, Decrement,
    Add(u8), Sub(u8),
    Write, Read,
    LoopBegin, LoopEnd,
    ResetCell,
    ScanCells(bool),
    TapeState,
}

#[derive(Clone, Debug)]
enum Instruction {
    Move(i32),
    Add(u8), Sub(u8),
    Write, Read,
    Loop(Vec<Instruction>),
    ResetCell,
    ScanCells(bool),
    TapeState,
}

/// this turns the source code into a sequence of opcodes.
/// should be somewhat easier to work with :3
fn tokenize(source: &str) -> Vec<OpCode> {
    source.chars().filter_map(|symbol| {
        match symbol {
            '>' => Some(OpCode::IncrementPointer),
            '<' => Some(OpCode::DecrementPointer),
            '+' => Some(OpCode::Increment),
            '-' => Some(OpCode::Decrement),
            '.' => Some(OpCode::Write),
            ',' => Some(OpCode::Read),
            '[' => Some(OpCode::LoopBegin),
            ']' => Some(OpCode::LoopEnd),
            '|' => Some(OpCode::TapeState), // additional, mostly for debug
            _ => None,
        }
    }).collect()
}

fn optimize_opcodes(opcodes: &mut Vec<OpCode>) {
    let mut i = 0;
    while i < opcodes.len() {
        match opcodes[i] {
            OpCode::LoopBegin if i + 2 < opcodes.len() 
            && (opcodes[i + 1] == OpCode::Decrement 
                    || opcodes[i + 1] == OpCode::Increment) 
                && opcodes[i + 2] == OpCode::LoopEnd => {
                opcodes.drain(i..i + 3);
                opcodes.insert(i, OpCode::ResetCell);
            }

            OpCode::LoopBegin if i + 2 < opcodes.len() && (opcodes[i + 1] == 
                OpCode::DecrementPointer 
                    || opcodes[i + 1] == OpCode::IncrementPointer) 
                && opcodes[i + 2] == OpCode::LoopEnd => {
                opcodes.drain(i..i + 3);
                opcodes.insert(i, OpCode::ScanCells(opcodes[i + 1] == 
                    OpCode::IncrementPointer));
            }
            
            OpCode::Increment | OpCode::Decrement => {
                let mut count = 1;
                let mut j = i + 1;
                while j < opcodes.len() && opcodes[j] == opcodes[i] {
                    count += 1;
                    j += 1;
                    }

                let replacement = match opcodes[i] {
                    OpCode::Increment => OpCode::Add(count),
                    OpCode::Decrement => OpCode::Sub(count),
                    _ => unreachable!(),
                };

                opcodes.drain(i..j);
                opcodes.insert(i, replacement);
                
            }

            OpCode::IncrementPointer | OpCode::DecrementPointer => {
                let mut offset = 0;
                let mut j = i;
                while j < opcodes.len() {
                    match opcodes[j] {
                        OpCode::IncrementPointer => offset += 1,
                        OpCode::DecrementPointer => offset -= 1,
                        _ => break,
                    }
                    j += 1;
                }
                if offset != 0 {
                    opcodes.drain(i..j);
                    opcodes.insert(i, OpCode::Move(offset));
                }
            }
            _ => (),
        }
        i += 1;
    }
}

fn parse(opcodes: Vec<OpCode>) -> Vec<Instruction> {
    let mut program: Vec<Instruction> = Vec::new();
    let mut loop_stack = 0;
    let mut loop_start = 0;

    for (i, op) in opcodes.iter().enumerate() {
        if loop_stack == 0 { // not inside a loop
            let instr = match op {

                OpCode::Move(offset)  => Some(Instruction::Move(*offset)),
                OpCode::Add(count) => Some(Instruction::Add(*count)),
                OpCode::Sub(count) => Some(Instruction::Sub(*count)),
                OpCode::Write               => Some(Instruction::Write),
                OpCode::Read                => Some(Instruction::Read),
                OpCode::ResetCell           => Some(Instruction::ResetCell),
                OpCode::ScanCells(bool) => Some(Instruction::ScanCells(*bool)),
                
                OpCode::LoopBegin => {
                    loop_start = i;
                    loop_stack += 1;
                    None
                },

                OpCode::LoopEnd => panic!("STRAY CLOSING BRACKET AT #{}!", i),

                OpCode::IncrementPointer    => None, // Instruction::Move is used instead
                OpCode::DecrementPointer    => None, // Instruction::Move is used instead
                OpCode::Increment           => None,
                OpCode::Decrement           => None,
                OpCode::TapeState           => Some(Instruction::TapeState),
            };

            if let Some(instr) = instr {
                program.push(instr);
            }

        } else {
            match op { //inside a loop
                OpCode::LoopBegin => loop_stack += 1,
                OpCode::LoopEnd => {
                    loop_stack -= 1;

                    if loop_stack == 0 {
                        program.push(Instruction::Loop(parse(opcodes[loop_start+1..i].to_vec())));
                    }
                },
                _ => (),
            }
        }
    }

    if loop_stack != 0 {
        panic!("STRAY OPENING BRACKET AT #{}!", loop_start);
    }

    program
}

/// executes a program that was previously parsed
fn execute(instructions: &Vec<Instruction>, tape: &mut [u8;30000], data_pointer: &mut usize) {
    for instr in instructions {
        match instr {
            
            Instruction::Move(offset) => {
                if *offset < 0 {
                    *data_pointer = data_pointer.wrapping_sub(offset.unsigned_abs() as usize) % tape.len();
                } else {
                    *data_pointer = data_pointer.wrapping_add(*offset as usize) % tape.len();
                }
            }
            Instruction::Add(count) => tape[*data_pointer] = 
                tape[*data_pointer].wrapping_add(*count),
            Instruction::Sub(count) => tape[*data_pointer] = 
                tape[*data_pointer].wrapping_sub(*count),
            Instruction::ResetCell => tape[*data_pointer] = 0,
            Instruction::ScanCells(direction) => {
                if *direction {
                    while tape[*data_pointer] != 0 {
                        *data_pointer += 1;
                    }
                } else {
                    while tape[*data_pointer] != 0 {
                        *data_pointer -= 1;
                    }
                }
            }
            Instruction::Write => print!("{}", tape[*data_pointer] as char),
            Instruction::Read => {
                let mut input: [u8; 1] = [0; 1];
                std::io::stdin().read_exact(&mut input).expect("FAILED TO READ 'stdin'!");
                tape[*data_pointer] = input[0];
            },
            Instruction::Loop(nested_instructions) => {
                while tape[*data_pointer] != 0 {
                    execute(&nested_instructions, tape, data_pointer)
                }
            }
            Instruction::TapeState => {
                let last_non_zero_index = tape.iter().rposition(|&x| x != 0).map(|i| i + 1).unwrap_or(0);
                for i in 0..last_non_zero_index {print!("{} ", i);}println!();
            }
        }
    }
}

use std::{env,time::Instant};
use std::fs::File;

fn main() {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();

    // Ensure there is at least 1 argument: the file path
    if args.len() < 2 || args.len() > 3 {
        eprintln!("Usage: <program> <file path> [<debug mode>]");
        return;
    }

    // Parse file path and optionally parse debug mode
    let file_path = &args[1];
    let debug_mode: u8 = if args.len() == 3 {
        match args[2].trim().parse() {
            Ok(num) => num,
            Err(_) => {
                eprintln!("Debug mode must be a number between 0 and 255");
                return;
            }
        }
    } else {
        0
    };

    // Read the content of the file
    let mut file_content = String::new();
    match File::open(file_path) {
        Ok(mut file) => {
            if let Err(e) = file.read_to_string(&mut file_content) {
                eprintln!("error reading file: {}\nfile path: {}", e,file_path);
                return;
            }
        }
        Err(e) => {
            eprintln!("error opening file: {}\nfile path:{}", e,file_path);
            return;
        }
    }

    // Filter the file content to include only the specified symbols
    let allowed_symbols = "><+-.,[]|";
    let filtered_content: String = file_content.chars()
        .filter(|c| allowed_symbols.contains(*c))
        .collect();
    // Print the filtered content and debug mode
    if debug_mode == 1u8 {
        println!("Filtered content: {}", filtered_content);
        println!("Debug mode: {}", debug_mode);

        }
    let source_code = filtered_content;

    // turn the source code into a vector of opcodes
    let mut opcodes = tokenize(&source_code);

    optimize_opcodes(&mut opcodes);
    if debug_mode == 1u8 {
        println!("Original Opcodes:");
        println!("{:?}",&opcodes);
        println!("Optimized Opcodes:");
        println!("{:?}",&opcodes);
    }

    // parse opcodes into a program / list of instructions
    let program = parse(opcodes);

    // set up thhings and run program
    let mut tape = [0u8; 30000];
    let mut data_pointer = 0;

    let start_time = Instant::now();
    
    execute(&program, &mut tape, &mut data_pointer);
    
    let elapsed_time = start_time.elapsed();

    println!("Execution took: {:?}", elapsed_time);
}
