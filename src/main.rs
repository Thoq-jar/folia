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

fn load_bytecode(input_files: &[String]) -> Result<Vec<u8>, String> {
    let first_file = &input_files[0];
    let path = Path::new(first_file);
    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    if extension == "s" || extension == "asm" {
        for file in input_files {
            let file_path = Path::new(file);
            let file_ext = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");
            if file_ext != "s" && file_ext != "asm" {
                return Err("All files must be assembly files when using multiple files".to_string());
            }
        }

        let source = read_and_concatenate_files(input_files)?;
        let mut assembler = Assembler::new();
        assembler.assemble(&source).map_err(|e| format!("Assembler error: {}", e))
    } else {
        if input_files.len() > 1 {
            return Err("Multiple files are not supported for bytecode (.fam) files".to_string());
        }
        fs::read(first_file).map_err(|e| format!("Error reading bytecode file {}: {}", first_file, e))
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <command> [file1] [file2] ...", args[0]);
        println!("Commands:");
        println!("  compile <source.asm> [source2.asm] ... - Compile assembly files to .fam bytecode");
        println!("  run <program.fam>                      - Run bytecode program");
        println!("  run <source.asm> [source2.asm] ...    - Compile and run assembly programs");
        println!("  debug <program.fam>                    - Run with debug output");
        println!("  debug <source.asm> [source2.asm] ...  - Compile and debug assembly programs");
        println!("  step <program.fam>                     - Interactive step debugger");
        println!("  step <source.asm> [source2.asm] ...   - Interactive step debugger");
        println!("  trace <program.fam>                    - Run with execution trace");
        println!("  trace <source.asm> [source2.asm] ...  - Run with execution trace");
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
        "run" | "debug" | "step" | "trace" => {
            if args.len() < 3 {
                println!("Usage: {} {} <program.fam|source.asm> [source2.asm] ...", args[0], command);
                process::exit(1);
            }

            let input_files: Vec<String> = args[2..].to_vec();
            let bytecode = match load_bytecode(&input_files) {
                Ok(data) => data,
                Err(e) => {
                    println!("{}", e);
                    process::exit(1);
                }
            };

            let mut vm = Runtime::new();
            vm.load_program(&bytecode);

            match command.as_str() {
                "run" => {
                    match vm.run() {
                        Ok(()) => {}
                        Err(error) => {
                            error.print_error();
                            process::exit(1);
                        }
                    }
                }
                "debug" => {
                    println!("Initial state:");
                    vm.debug_state();
                    println!("\nRunning program...\n");

                    match vm.run() {
                        Ok(()) => {
                            println!("\nFinal state:");
                            vm.debug_state();
                        }
                        Err(error) => {
                            error.print_error();
                            println!("\nVM state at error:");
                            vm.debug_state();
                            process::exit(1);
                        }
                    }
                }
                "step" => {
                    println!("Interactive Step Debugger Started");
                    vm.debug_state();

                    loop {
                        match vm.debug_step() {
                            Ok(true) => {
                                match vm.run() {
                                    Ok(()) => break,
                                    Err(error) => {
                                        error.print_error();
                                        vm.debug_state();
                                        break;
                                    }
                                }
                            }
                            Ok(false) => {
                                println!("Program terminated");
                                break;
                            }
                            Err(error) => {
                                error.print_error();
                                vm.debug_state();
                                break;
                            }
                        }
                    }
                }
                "trace" => {
                    println!("📊 Execution Trace Mode");
                    match vm.run_with_trace() {
                        Ok(()) => {
                            println!("\nProgram completed");
                            vm.debug_performance();
                        }
                        Err(error) => {
                            error.print_error();
                            vm.debug_state();
                            process::exit(1);
                        }
                    }
                }
                _ => unreachable!()
            }
        }
        _ => {
            println!("Unknown command: {}", command);
            println!("Available commands: compile, run, debug, step, trace");
            process::exit(1);
        }
    }
}