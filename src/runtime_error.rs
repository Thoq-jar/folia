use crate::runtime::{Instruction, StackFrame};

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub message: String,
    pub pc: usize,
    pub instruction: Instruction,
    pub stack_trace: Vec<StackFrame>,
}

impl RuntimeError {
    pub fn new(message: String, pc: usize, instruction: Instruction, stack_trace: Vec<StackFrame>) -> Self {
        RuntimeError {
            message,
            pc,
            instruction,
            stack_trace,
        }
    }

    pub fn print_error(&self) {
        eprintln!("Runtime Error: {}", self.message);
        eprintln!("  at PC: {} (instruction: {:?})", self.pc / 2, self.instruction.opcode);

        if !self.stack_trace.is_empty() {
            eprintln!("\nStack trace:");
            for (i, frame) in self.stack_trace.iter().rev().enumerate() {
                eprintln!("  #{}: PC {} - {}", i, frame.pc / 2, frame.instruction);
            }
        }

        eprintln!("\nInstruction details:");
        eprintln!("  Opcode: {:?}", self.instruction.opcode);
        eprintln!("  Registers: rd={}, rs1={}, rs2={}",
                  self.instruction.rd, self.instruction.rs1, self.instruction.rs2);
        eprintln!("  Immediate: {}", self.instruction.immediate);
    }
}
