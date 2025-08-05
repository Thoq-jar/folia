use std::io;
use std::io::{BufWriter, Write};
use crate::op_codes::OpCode;
use crate::runtime_error::RuntimeError;

#[derive(Debug, Clone)]
pub struct Instruction {
    pub(crate) opcode: OpCode,
    pub(crate) rd: u8,
    pub(crate) rs1: u8,
    pub(crate) rs2: u8,
    pub(crate) immediate: i32,
    pub(crate) label: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StackFrame {
    pub pc: usize,
    pub instruction: String,
}

pub struct Runtime {
    registers: [i32; 32],
    memory: [i32; 1024],
    stack: Vec<i32>,
    pc: usize,
    flags: Flags,
    running: bool,
    call_stack: Vec<usize>,
    instruction_count: usize,
    output_buffer: BufWriter<io::Stdout>,
}

#[derive(Debug, Clone, Copy)]
struct Flags {
    zero: bool,
    negative: bool,
    carry: bool,
    overflow: bool,
}

impl Runtime {
    pub(crate) fn new() -> Self {
        Runtime {
            registers: [0; 32],
            memory: [0; 1024],
            stack: Vec::new(),
            pc: 0,
            flags: Flags {
                zero: false,
                negative: false,
                carry: false,
                overflow: false,
            },
            running: true,
            call_stack: Vec::new(),
            instruction_count: 0,
            output_buffer: BufWriter::new(io::stdout()),
        }
    }

    #[inline]
    fn set_flags(&mut self, value: i32) {
        self.flags.zero = value == 0;
        self.flags.negative = value < 0;
    }

    fn create_stack_trace(&self) -> Vec<StackFrame> {
        let mut stack_trace = Vec::new();

        if let Some(current_instruction) = self.get_instruction_at_pc(self.pc - 2) {
            stack_trace.push(StackFrame {
                pc: self.pc - 2,
                instruction: format!("{:?}", current_instruction.opcode),
            });
        }

        for &call_pc in &self.call_stack {
            if let Some(instruction) = self.get_instruction_at_pc(call_pc) {
                stack_trace.push(StackFrame {
                    pc: call_pc,
                    instruction: format!("{:?}", instruction.opcode),
                });
            }
        }

        stack_trace
    }

    fn get_instruction_at_pc(&self, pc: usize) -> Option<Instruction> {
        if pc >= self.memory.len() - 1 {
            return None;
        }

        let word1 = self.memory[pc];
        let word2 = self.memory[pc + 1];

        let opcode = OpCode::from_u8(((word1 >> 24) & 0xFF) as u8)?;
        let rd = ((word1 >> 16) & 0xFF) as u8;
        let rs1 = ((word1 >> 8) & 0xFF) as u8;
        let rs2 = (word1 & 0xFF) as u8;
        let immediate = word2;

        Some(Instruction {
            opcode,
            rd,
            rs1,
            rs2,
            immediate,
            label: None,
        })
    }

    fn runtime_error(&self, message: String, instruction: Instruction) -> RuntimeError {
        RuntimeError::new(
            message,
            self.pc - 2,
            instruction,
            self.create_stack_trace(),
        )
    }

    pub(crate) fn load_program(&mut self, bytecode: &[u8]) {
        let mut data_end = 0;
        for i in (0..bytecode.len()).step_by(8) {
            if i + 7 < bytecode.len() {
                let chunk = &bytecode[i..i + 8];
                if chunk == [0, 0, 0, 0, 0, 0, 0, 0] {
                    data_end = i;
                    break;
                }
            }
        }

        for (i, &byte) in bytecode[..data_end].iter().enumerate() {
            if i + 512 < self.memory.len() {
                let addr = (i + 512) / 4;
                let offset = (i + 512) % 4;
                let current = self.memory[addr];
                let mask = !(0xFF << (offset * 8));
                let new_val = (current & mask) | ((byte as i32) << (offset * 8));
                self.memory[addr] = new_val;
            }
        }

        if data_end + 8 <= bytecode.len() {
            let start_bytes = &bytecode[data_end..data_end + 4];
            let start_pc = u32::from_le_bytes([
                start_bytes[0],
                start_bytes[1],
                start_bytes[2],
                start_bytes[3],
            ]) as usize;
            self.pc = start_pc * 2;

            let instructions_start = data_end + 8;
            for (i, chunk) in bytecode[instructions_start..].chunks(8).enumerate() {
                if chunk.len() == 8 {
                    let addr = i * 2;
                    if addr < self.memory.len() {
                        let opcode = chunk[0];
                        let rd = chunk[1];
                        let rs1 = chunk[2];
                        let rs2 = chunk[3];
                        let immediate =
                            i32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);

                        self.memory[addr] = ((opcode as i32) << 24)
                            | ((rd as i32) << 16)
                            | ((rs1 as i32) << 8)
                            | (rs2 as i32);
                        self.memory[addr + 1] = immediate;
                    }
                }
            }
        }
    }

    #[inline]
    fn fetch(&mut self) -> Option<Instruction> {
        if self.pc >= self.memory.len() - 1 {
            return None;
        }

        let word1 = self.memory[self.pc];
        let word2 = self.memory[self.pc + 1];

        let opcode = OpCode::from_u8(((word1 >> 24) & 0xFF) as u8)?;
        let rd = ((word1 >> 16) & 0xFF) as u8;
        let rs1 = ((word1 >> 8) & 0xFF) as u8;
        let rs2 = (word1 & 0xFF) as u8;
        let immediate = word2;

        self.pc += 2;
        self.instruction_count += 1;

        Some(Instruction {
            opcode,
            rd,
            rs1,
            rs2,
            immediate,
            label: None,
        })
    }

    #[inline]
    fn execute(&mut self, instruction: Instruction) -> Result<(), RuntimeError> {
        match instruction.opcode {
            OpCode::MOV => {
                if instruction.rs1 != 0 {
                    self.registers[instruction.rd as usize] =
                        self.registers[instruction.rs1 as usize];
                } else {
                    self.registers[instruction.rd as usize] = instruction.immediate;
                }
            }
            OpCode::ADD => {
                let val1 = self.registers[instruction.rs1 as usize];
                let val2 = if instruction.rs2 != 0 {
                    self.registers[instruction.rs2 as usize]
                } else {
                    instruction.immediate
                };
                self.registers[instruction.rd as usize] = val1.wrapping_add(val2);
            }
            OpCode::SUB => {
                let val1 = self.registers[instruction.rs1 as usize];
                let val2 = if instruction.rs2 != 0 {
                    self.registers[instruction.rs2 as usize]
                } else {
                    instruction.immediate
                };
                self.registers[instruction.rd as usize] = val1.wrapping_sub(val2);
            }
            OpCode::MUL => {
                let val1 = self.registers[instruction.rs1 as usize];
                let val2 = if instruction.rs2 != 0 {
                    self.registers[instruction.rs2 as usize]
                } else {
                    instruction.immediate
                };
                self.registers[instruction.rd as usize] = val1.wrapping_mul(val2);
            }
            OpCode::DIV => {
                let val1 = self.registers[instruction.rs1 as usize];
                let val2 = if instruction.rs2 != 0 {
                    self.registers[instruction.rs2 as usize]
                } else {
                    instruction.immediate
                };

                if val2 == 0 {
                    return Err(self.runtime_error(
                        "Division by zero".to_string(),
                        instruction,
                    ));
                }

                let result = val1 / val2;
                self.registers[instruction.rd as usize] = result;
                self.set_flags(result);
            }
            OpCode::CMP => {
                let val1 = self.registers[instruction.rs1 as usize];
                let val2 = if instruction.rs2 != 0 {
                    self.registers[instruction.rs2 as usize]
                } else {
                    instruction.immediate
                };
                let result = val1 - val2;
                self.set_flags(result);
            }
            OpCode::JMP => {
                self.pc = (instruction.immediate as usize) * 2;
            }
            OpCode::JEQ => {
                if self.flags.zero {
                    self.pc = (instruction.immediate as usize) * 2;
                }
            }
            OpCode::JNE => {
                if !self.flags.zero {
                    self.pc = (instruction.immediate as usize) * 2;
                }
            }
            OpCode::JLT => {
                if self.flags.negative {
                    self.pc = (instruction.immediate as usize) * 2;
                }
            }
            OpCode::JGT => {
                if !self.flags.negative && !self.flags.zero {
                    self.pc = (instruction.immediate as usize) * 2;
                }
            }
            OpCode::LOAD => {
                let addr = if instruction.rs1 != 0 {
                    self.registers[instruction.rs1 as usize] as usize
                } else {
                    instruction.immediate as usize
                };

                if addr >= self.memory.len() * 4 {
                    return Err(self.runtime_error(
                        format!("Memory access out of bounds: address {} (max: {})",
                                addr, self.memory.len() * 4 - 1),
                        instruction,
                    ));
                }

                let word_addr = addr / 4;
                let byte_offset = addr % 4;
                let word = self.memory[word_addr];
                let byte = (word >> (byte_offset * 8)) & 0xFF;
                self.registers[instruction.rd as usize] = byte;
            }
            OpCode::STORE => {
                let addr = if instruction.rs1 != 0 {
                    self.registers[instruction.rs1 as usize] as usize
                } else {
                    instruction.immediate as usize
                };

                if addr >= self.memory.len() {
                    return Err(self.runtime_error(
                        format!("Memory access out of bounds: address {} (max: {})",
                                addr, self.memory.len() - 1),
                        instruction,
                    ));
                }

                self.memory[addr] = self.registers[instruction.rd as usize];
            }
            OpCode::PUSH => {
                self.stack.push(self.registers[instruction.rd as usize]);
            }
            OpCode::POP => {
                if let Some(value) = self.stack.pop() {
                    self.registers[instruction.rd as usize] = value;
                } else {
                    return Err(self.runtime_error(
                        "Stack underflow: attempted to pop from empty stack".to_string(),
                        instruction,
                    ));
                }
            }
            OpCode::CALL => {
                self.call_stack.push(self.pc);
                self.stack.push(self.pc as i32);
                self.pc = (instruction.immediate as usize) * 2;
            }
            OpCode::RET => {
                if let Some(addr) = self.stack.pop() {
                    self.pc = addr as usize;
                    self.call_stack.pop();
                } else {
                    return Err(self.runtime_error(
                        "Stack underflow: attempted to return with empty stack".to_string(),
                        instruction,
                    ));
                }
            }
            OpCode::HALT => {
                self.output_buffer.flush().unwrap();
                self.running = false;
            }
            OpCode::NOP => {}
            OpCode::AND => {
                let val1 = self.registers[instruction.rs1 as usize];
                let val2 = if instruction.rs2 != 0 {
                    self.registers[instruction.rs2 as usize]
                } else {
                    instruction.immediate
                };
                let result = val1 & val2;
                self.registers[instruction.rd as usize] = result;
                self.set_flags(result);
            }
            OpCode::OR => {
                let val1 = self.registers[instruction.rs1 as usize];
                let val2 = if instruction.rs2 != 0 {
                    self.registers[instruction.rs2 as usize]
                } else {
                    instruction.immediate
                };
                let result = val1 | val2;
                self.registers[instruction.rd as usize] = result;
                self.set_flags(result);
            }
            OpCode::XOR => {
                let val1 = self.registers[instruction.rs1 as usize];
                let val2 = if instruction.rs2 != 0 {
                    self.registers[instruction.rs2 as usize]
                } else {
                    instruction.immediate
                };
                let result = val1 ^ val2;
                self.registers[instruction.rd as usize] = result;
                self.set_flags(result);
            }
            OpCode::NOT => {
                let val = self.registers[instruction.rs1 as usize];
                let result = !val;
                self.registers[instruction.rd as usize] = result;
                self.set_flags(result);
            }
            OpCode::LSL => {
                let val = self.registers[instruction.rs1 as usize];
                let shift = instruction.immediate as u32;

                if shift >= 32 {
                    return Err(self.runtime_error(
                        format!("Invalid left shift: shift amount {} >= 32", shift),
                        instruction,
                    ));
                }

                let result = val << shift;
                self.registers[instruction.rd as usize] = result;
                self.set_flags(result);
            }
            OpCode::LSR => {
                let val = self.registers[instruction.rs1 as usize] as u32;
                let shift = instruction.immediate as u32;

                if shift >= 32 {
                    return Err(self.runtime_error(
                        format!("Invalid right shift: shift amount {} >= 32", shift),
                        instruction,
                    ));
                }

                let result = (val >> shift) as i32;
                self.registers[instruction.rd as usize] = result;
                self.set_flags(result);
            }
            OpCode::PRINT => {
                let value = self.registers[instruction.rd as usize];
                write!(self.output_buffer, "{}", value).unwrap();
            }
            OpCode::PRINTC => {
                let value = self.registers[instruction.rd as usize] as u8;
                if value == 0 {
                    let mut addr = 512;
                    loop {
                        if addr >= self.memory.len() * 4 {
                            break;
                        }
                        let word_addr = addr / 4;
                        let byte_offset = addr % 4;
                        let word = self.memory[word_addr];
                        let byte = ((word >> (byte_offset * 8)) & 0xFF) as u8;
                        if byte == 0 {
                            break;
                        }
                        write!(self.output_buffer, "{}", byte as char).unwrap();
                        addr += 1;
                    }
                } else {
                    write!(self.output_buffer, "{}", value as char).unwrap();
                }
            }
            OpCode::INPUT => {
                self.output_buffer.flush().unwrap();
                let mut input = String::new();
                io::stdout().flush().unwrap();

                if io::stdin().read_line(&mut input).is_ok() {
                    let trimmed = input.trim();
                    match trimmed.parse::<i32>() {
                        Ok(value) => {
                            self.registers[instruction.rd as usize] = value;
                        }
                        Err(_) => {
                            return match trimmed.parse::<i64>() {
                                Ok(big_value) => {
                                    Err(self.runtime_error(
                                        format!("Input integer overflow: {} is outside the range of 32-bit signed integers ({} to {})",
                                                big_value, i32::MIN, i32::MAX),
                                        instruction,
                                    ))
                                }
                                Err(_) => {
                                    Err(self.runtime_error(
                                        format!("Invalid integer input: '{}' is not a valid integer", trimmed),
                                        instruction,
                                    ))
                                }
                            }
                        }
                    }
                } else {
                    return Err(self.runtime_error(
                        "Failed to read input from stdin".to_string(),
                        instruction,
                    ));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn run(&mut self) -> Result<(), RuntimeError> {
        while self.running {
            if let Some(instruction) = self.fetch() {
                if let Err(error) = self.execute(instruction) {
                    return Err(error);
                }
            } else {
                break;
            }
        }
        Ok(())
    }

    pub(crate) fn debug_state(&self) {
        println!("PC: {} (instruction #{})", self.pc / 2, self.instruction_count);
        println!("Registers:");
        for (i, &val) in self.registers.iter().enumerate().take(8) {
            println!("  r{}: {}", i, val);
        }
        println!(
            "Flags: Z={} N={} C={} V={}",
            self.flags.zero, self.flags.negative, self.flags.carry, self.flags.overflow
        );
        println!("Stack size: {}, Call stack depth: {}", self.stack.len(), self.call_stack.len());
    }
}
