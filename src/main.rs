mod op_codes;
mod runtime_error;
mod assembler;
mod runtime;

use std::env;
use std::fs;
use std::process;
use std::path::Path;
use crate::assembler::Assembler;
use crate::runtime::Runtime;

fn read_and_concatenate_files(files: &[String]) -> Result<String, String> {
    let mut combined_source = String::new();
    
    for (i, file) in files.iter().enumerate() {
        let content = fs::read_to_string(file)
            .map_err(|e| format!("Error reading file {}: {}", file, e))?;
        
        if i > 0 {
            combined_source.push('\n');
        }
        combined_source.push_str(&content);
    }
    
    Ok(combined_source)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <command> [file1] [file2] ...", args[0]);
        println!("Commands:");
        println!("  compile <source.asm> [source2.asm] ... - Compile assembly files to .fam bytecode");
        println!("  run <program.fam>                      - Run bytecode program");
        println!("  run <source.asm> [source2.asm] ...    - Compile and run assembly programs (no .fam file saved)");
        println!("  debug <program.fam>                    - Run with debug output");
        println!("  debug <source.asm> [source2.asm] ...  - Compile and debug assembly programs (no .fam file saved)");
        process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "compile" => {
            if args.len() < 3 {
                println!("Usage: {} compile <source.asm> [source2.asm] ...", args[0]);
                process::exit(1);
            }

            let source_files: Vec<String> = args[2..].to_vec();
            let source = match read_and_concatenate_files(&source_files) {
                Ok(content) => content,
                Err(e) => {
                    println!("{}", e);
                    process::exit(1);
                }
            };

            let mut assembler = Assembler::new();
            match assembler.assemble(&source) {
                Ok(bytecode) => {
                    let output_file = if source_files.len() == 1 {
                        source_files[0].replace(".asm", ".fam").replace(".s", ".fam")
                    } else {
                        "program.fam".to_string()
                    };
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
                println!("Usage: {} {} <program.fam|source.asm> [source2.asm] ...", args[0], command);
                process::exit(1);
            }

            let input_files: Vec<String> = args[2..].to_vec();
            let first_file = &input_files[0];
            let path = Path::new(first_file);
            let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

            let bytecode = if extension == "s" || extension == "asm" {
                for file in &input_files {
                    let file_path = Path::new(file);
                    let file_ext = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");
                    if file_ext != "s" && file_ext != "asm" {
                        println!("Error: All files must be assembly files when using multiple files");
                        process::exit(1);
                    }
                }

                let source = match read_and_concatenate_files(&input_files) {
                    Ok(content) => content,
                    Err(e) => {
                        println!("{}", e);
                        process::exit(1);
                    }
                };

                let mut assembler = Assembler::new();
                match assembler.assemble(&source) {
                    Ok(bytecode) => bytecode,
                    Err(e) => {
                        println!("Assembler error: {}", e);
                        process::exit(1);
                    }
                }
            } else {
                if input_files.len() > 1 {
                    println!("Error: Multiple files are not supported for bytecode (.fam) files");
                    process::exit(1);
                }
                match fs::read(first_file) {
                    Ok(data) => data,
                    Err(e) => {
                        println!("Error reading bytecode file {}: {}", first_file, e);
                        process::exit(1);
                    }
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