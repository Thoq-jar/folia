mod op_codes;
mod runtime_error;
mod assembler;
mod runtime;

use std::env;
use std::fs;
use std::process;
use crate::assembler::Assembler;
use crate::runtime::Runtime;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <command> [file]", args[0]);
        println!("Commands:");
        println!("  compile <source.asm> - Compile assembly to .fam bytecode");
        println!("  run <program.fam>    - Run bytecode program");
        println!("  debug <program.fam>  - Run with debug output");
        process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "compile" => {
            if args.len() < 3 {
                println!("Usage: {} compile <source.asm>", args[0]);
                process::exit(1);
            }

            let source_file = &args[2];
            let source = match fs::read_to_string(source_file) {
                Ok(content) => content,
                Err(e) => {
                    println!("Error reading file {}: {}", source_file, e);
                    process::exit(1);
                }
            };

            let mut assembler = Assembler::new();
            match assembler.assemble(&source) {
                Ok(bytecode) => {
                    let output_file = source_file.replace(".asm", ".fam").replace(".s", ".fam");
                    match fs::write(&output_file, &bytecode) {
                        Ok(_) => println!("Compiled to {}", output_file),
                        Err(e) => {
                            println!("Error writing bytecode: {}", e);
                            process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    println!("Assembly error: {}", e);
                    process::exit(1);
                }
            }
        }
        "run" | "debug" => {
            if args.len() < 3 {
                println!("Usage: {} {} <program.fam>", args[0], command);
                process::exit(1);
            }

            let bytecode_file = &args[2];
            let bytecode = match fs::read(bytecode_file) {
                Ok(data) => data,
                Err(e) => {
                    println!("Error reading bytecode file {}: {}", bytecode_file, e);
                    process::exit(1);
                }
            };

            let mut vm = Runtime::new();
            vm.load_program(&bytecode);

            if command == "debug" {
                println!("Initial state:");
                vm.debug_state();
                println!("\nRunning program...\n");
            }

            match vm.run() {
                Ok(()) => {
                    if command == "debug" {
                        println!("\nFinal state:");
                        vm.debug_state();
                    }
                }
                Err(error) => {
                    error.print_error();

                    if command == "debug" {
                        println!("\nVM state at error:");
                        vm.debug_state();
                    }

                    process::exit(1);
                }
            }
        }
        _ => {
            println!("Unknown command: {}", command);
            println!("Available commands: compile, run, debug");
            process::exit(1);
        }
    }
}