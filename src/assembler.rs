use std::collections::HashMap;
use crate::op_codes::OpCode;
use crate::runtime::Instruction;

pub struct Assembler {
    labels: HashMap<String, usize>,
    instructions: Vec<Instruction>,
    data_section: Vec<u8>,
    start_label: Option<String>,
    current_section: Section,
}

#[derive(Debug, Clone, PartialEq)]
enum Section {
    Text,
    Data,
}

impl Assembler {
    pub(crate) fn new() -> Self {
        Assembler {
            labels: HashMap::new(),
            instructions: Vec::new(),
            data_section: Vec::new(),
            start_label: None,
            current_section: Section::Text,
        }
    }

    #[inline]
    fn parse_register(reg: &str) -> Option<u8> {
        if reg.starts_with('r') || reg.starts_with('R') {
            reg[1..].parse().ok()
        } else {
            None
        }
    }

    #[inline]
    fn parse_immediate(imm: &str) -> Option<i32> {
        if imm.starts_with('#') {
            imm[1..].parse().ok()
        } else if imm.starts_with("0x") {
            i32::from_str_radix(&imm[2..], 16).ok()
        } else {
            imm.parse().ok()
        }
    }

    fn parse_instruction_parts(line: &str) -> Vec<String> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut in_brackets = false;

        for ch in line.chars() {
            match ch {
                '[' => {
                    in_brackets = true;
                    current.push(ch);
                }
                ']' => {
                    in_brackets = false;
                    current.push(ch);
                }
                ',' if !in_brackets => {
                    if !current.trim().is_empty() {
                        parts.push(current.trim().to_string());
                        current.clear();
                    }
                }
                ch if ch.is_whitespace() && !in_brackets => {
                    if !current.trim().is_empty() {
                        parts.push(current.trim().to_string());
                        current.clear();
                    }
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        if !current.trim().is_empty() {
            parts.push(current.trim().to_string());
        }

        parts
    }

    pub(crate) fn assemble(&mut self, source: &str) -> Result<Vec<u8>, String> {
        let lines: Vec<&str> = source.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let mut line = line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }

            if line == ".data" {
                self.current_section = Section::Data;
                continue;
            }

            if line == ".text" {
                self.current_section = Section::Text;
                continue;
            }

            if line.starts_with(".start") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    self.start_label = Some(parts[1].to_string());
                }
                continue;
            }

            if let Some(colon_pos) = line.find(':') {
                let (label_part, code_part) = line.split_at(colon_pos);
                let label = label_part.trim().to_string();

                if !label.contains(char::is_whitespace) {
                    match self.current_section {
                        Section::Text => {
                            self.labels.insert(label, self.instructions.len());
                        }
                        Section::Data => {
                            self.labels.insert(label, 512 + self.data_section.len());
                        }
                    }

                    line = code_part[1..].trim();
                    if line.is_empty() {
                        continue;
                    }
                }
            }

            if self.current_section == Section::Data {
                if line.starts_with(".ascii") || line.starts_with(".string") {
                    let start = line.find('"').ok_or("Missing opening quote")?;
                    let end = line.rfind('"').ok_or("Missing closing quote")?;
                    if start >= end {
                        return Err("Invalid string format".to_string());
                    }
                    let string_content = &line[start + 1..end];
                    let processed = string_content
                        .replace("\\n", "\n")
                        .replace("\\t", "\t")
                        .replace("\\r", "\r")
                        .replace("\\\\", "\\")
                        .replace("\\\"", "\"");

                    self.data_section.extend(processed.bytes());
                    if line.starts_with(".string") {
                        self.data_section.push(0);
                    }
                    continue;
                }

                if line.starts_with(".byte") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    for part in &parts[1..] {
                        let byte_val = part
                            .trim_end_matches(',')
                            .parse::<u8>()
                            .map_err(|_| format!("Invalid byte value: {}", part))?;
                        self.data_section.push(byte_val);
                    }
                    continue;
                }

                if line.starts_with(".word") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    for part in &parts[1..] {
                        let word_val = part
                            .trim_end_matches(',')
                            .parse::<i32>()
                            .map_err(|_| format!("Invalid word value: {}", part))?;
                        self.data_section.extend_from_slice(&word_val.to_le_bytes());
                    }
                    continue;
                }
                continue;
            }

            let parts = Self::parse_instruction_parts(line);
            if parts.is_empty() {
                continue;
            }

            let opcode_str = parts[0].to_uppercase();
            let opcode = match opcode_str.to_uppercase().as_str() {
                "MOV" => OpCode::MOV,
                "ADD" => OpCode::ADD,
                "SUB" => OpCode::SUB,
                "MUL" => OpCode::MUL,
                "DIV" => OpCode::DIV,
                "CMP" => OpCode::CMP,
                "JMP" => OpCode::JMP,
                "JEQ" => OpCode::JEQ,
                "JNE" => OpCode::JNE,
                "JLT" => OpCode::JLT,
                "JGT" => OpCode::JGT,
                "LEA" => OpCode::LEA,
                "STO" => OpCode::STORE,
                "PUS" => OpCode::PUSH,
                "POP" => OpCode::POP,
                "CAL" => OpCode::CALL,
                "RET" => OpCode::RET,
                "HLT" => OpCode::HALT,
                "NOP" => OpCode::NOP,
                "AND" => OpCode::AND,
                "OR" => OpCode::OR,
                "XOR" => OpCode::XOR,
                "NOT" => OpCode::NOT,
                "LSL" => OpCode::LSL,
                "LSR" => OpCode::LSR,
                "PRT" => OpCode::PRINT,
                "PRC" => OpCode::PRINTC,
                "INP" => OpCode::INPUT,
                _ => {
                    return Err(format!(
                        "Unknown opcode: {} at line {}",
                        opcode_str,
                        line_num + 1
                    ));
                }
            };

            let mut instruction = Instruction {
                opcode,
                rd: 0,
                rs1: 0,
                rs2: 0,
                immediate: 0,
                label: None,
            };

            match opcode {
                OpCode::MOV => {
                    if parts.len() >= 3 {
                        instruction.rd = Self::parse_register(&parts[1])
                            .ok_or_else(|| format!("Invalid register: {}", parts[1]))?;
                        if let Some(imm) = Self::parse_immediate(&parts[2]) {
                            instruction.immediate = imm;
                        } else if let Some(reg) = Self::parse_register(&parts[2]) {
                            instruction.rs1 = reg;
                        } else {
                            instruction.label = Some(parts[2].clone());
                        }
                    } else {
                        return Err(format!("MOV instruction requires comma-separated operands at line {}", line_num + 1));
                    }
                }
                OpCode::ADD
                | OpCode::SUB
                | OpCode::MUL
                | OpCode::DIV
                | OpCode::AND
                | OpCode::OR
                | OpCode::XOR => {
                    if parts.len() >= 4 {
                        instruction.rd = Self::parse_register(&parts[1])
                            .ok_or_else(|| format!("Invalid register: {}", parts[1]))?;
                        instruction.rs1 = Self::parse_register(&parts[2])
                            .ok_or_else(|| format!("Invalid register: {}", parts[2]))?;
                        if let Some(imm) = Self::parse_immediate(&parts[3]) {
                            instruction.immediate = imm;
                        } else if let Some(reg) = Self::parse_register(&parts[3]) {
                            instruction.rs2 = reg;
                        }
                    } else {
                        return Err(format!("Instruction {} requires comma-separated operands at line {}", opcode_str, line_num + 1));
                    }
                }
                OpCode::CMP => {
                    if parts.len() >= 3 {
                        instruction.rs1 = Self::parse_register(&parts[1])
                            .ok_or_else(|| format!("Invalid register: {}", parts[1]))?;
                        if let Some(imm) = Self::parse_immediate(&parts[2]) {
                            instruction.immediate = imm;
                        } else if let Some(reg) = Self::parse_register(&parts[2]) {
                            instruction.rs2 = reg;
                        }
                    } else {
                        return Err(format!("CMP instruction requires comma-separated operands at line {}", line_num + 1));
                    }
                }
                OpCode::JMP
                | OpCode::JEQ
                | OpCode::JNE
                | OpCode::JLT
                | OpCode::JGT
                | OpCode::CALL => {
                    if parts.len() >= 2 {
                        if let Some(imm) = Self::parse_immediate(&parts[1]) {
                            instruction.immediate = imm;
                        } else {
                            instruction.label = Some(parts[1].clone());
                        }
                    }
                }
                OpCode::LEA | OpCode::STORE => {
                    if parts.len() >= 3 {
                        instruction.rd = Self::parse_register(&parts[1])
                            .ok_or_else(|| format!("Invalid register: {}", parts[1]))?;
                        let addr_part = parts[2].trim_start_matches('[').trim_end_matches(']');
                        if let Some(reg) = Self::parse_register(addr_part) {
                            instruction.rs1 = reg;
                        } else if let Some(imm) = Self::parse_immediate(addr_part) {
                            instruction.immediate = imm;
                        } else {
                            instruction.label = Some(addr_part.to_string());
                        }
                    } else {
                        return Err(format!("Instruction {} requires comma-separated operands at line {}", opcode_str, line_num + 1));
                    }
                }
                OpCode::PUSH | OpCode::POP | OpCode::PRINT | OpCode::PRINTC | OpCode::INPUT => {
                    if parts.len() >= 2 {
                        instruction.rd = Self::parse_register(&parts[1])
                            .ok_or_else(|| format!("Invalid register: {}", parts[1]))?;
                    }
                }
                OpCode::NOT => {
                    if parts.len() >= 3 {
                        instruction.rd = Self::parse_register(&parts[1])
                            .ok_or_else(|| format!("Invalid register: {}", parts[1]))?;
                        instruction.rs1 = Self::parse_register(&parts[2])
                            .ok_or_else(|| format!("Invalid register: {}", parts[2]))?;
                    } else {
                        return Err(format!("NOT instruction requires comma-separated operands at line {}", line_num + 1));
                    }
                }
                OpCode::LSL | OpCode::LSR => {
                    if parts.len() >= 4 {
                        instruction.rd = Self::parse_register(&parts[1])
                            .ok_or_else(|| format!("Invalid register: {}", parts[1]))?;
                        instruction.rs1 = Self::parse_register(&parts[2])
                            .ok_or_else(|| format!("Invalid register: {}", parts[2]))?;
                        instruction.immediate = Self::parse_immediate(&parts[3])
                            .ok_or_else(|| format!("Invalid immediate: {}", parts[3]))?;
                    } else {
                        return Err(format!("Instruction {} requires comma-separated operands at line {}", opcode_str, line_num + 1));
                    }
                }
                _ => {}
            }

            self.instructions.push(instruction);
        }

        for instruction in &mut self.instructions {
            if let Some(label_name) = &instruction.label {
                if let Some(&addr) = self.labels.get(label_name) {
                    instruction.immediate = addr as i32;
                } else {
                    return Err(format!("Undefined label: {}", label_name));
                }
            }
        }

        let mut bytecode = Vec::new();
        bytecode.extend_from_slice(&self.data_section);

        while bytecode.len() % 8 != 0 {
            bytecode.push(0);
        }

        let start_pc = if let Some(ref start) = self.start_label {
            self.labels.get(start).copied().unwrap_or(0)
        } else {
            0
        };

        bytecode.extend_from_slice(&(start_pc as u32).to_le_bytes());
        bytecode.extend_from_slice(&[0u8; 4]);

        for instruction in &self.instructions {
            bytecode.push(instruction.opcode as u8);
            bytecode.push(instruction.rd);
            bytecode.push(instruction.rs1);
            bytecode.push(instruction.rs2);
            bytecode.extend_from_slice(&instruction.immediate.to_le_bytes());
        }

        Ok(bytecode)
    }
}