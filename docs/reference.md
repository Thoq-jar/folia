# Folia Runtime Documentation

Also see [Folia Virtual Machine - Call Reference](register_architecture.md)

```rust
use std::io;
use std::io::{BufWriter, Write};
use crate::op_codes::OpCode;
use crate::runtime_error::RuntimeError;

// Core data structures
pub struct Runtime {
    registers: [i32; 32],                   // 32 general-purpose registers
    memory: [i32; 1024],                    // 4KB memory space (1024 * 4 bytes)
    stack: Vec<i32>,                        // Runtime stack for function calls
    pc: usize,                              // Program counter
    flags: Flags,                           // Processor flags (zero, negative, carry, overflow)
    running: bool,                          // VM execution state
    call_stack: Vec<usize>,                 // Call stack for debugging
    instruction_count: usize,               // Instruction execution counter
    output_buffer: BufWriter<io::Stdout>,   // Buffered output for performance
}

// Processor flags for conditional operations
struct Flags {
    zero: bool,                             // Result was zero
    negative: bool,                         // Result was negative
    carry: bool,                            // Arithmetic carry occurred
    overflow: bool,                         // Arithmetic overflow occurred
}

// Instruction format
pub struct Instruction {
    opcode: OpCode,                         // Operation code
    rd: u8,                                 // Destination register
    rs1: u8,                                // Source register 1
    rs2: u8,                                // Source register 2
    immediate: i32,                         // Immediate value
    label: Option<String>,                  // Optional label for debugging
}
```


## System Calls and Core Functionality

### Memory Management

**`load_program(bytecode: &[u8])`**
- Loads compiled bytecode into VM memory
- Separates data section from instruction section
- Sets initial program counter from bytecode header
- Memory layout:
    - 0-511: Reserved
    - 512+: Data section
    - Instructions stored as 8-byte chunks (opcode + immediate)

**`LOAD` instruction**
- Loads byte from memory address into register
- Address can be from register or immediate value
- Bounds checking prevents memory access violations
- Format: `LOAD rd, [rs1]` or `LOAD rd, #immediate`

**`STORE` instruction**
- Stores register value to memory address
- Address specified by register or immediate
- Format: `STORE rd, [addr]`

### Register Operations

**`MOV` instruction**
- Move data between registers or load immediate values
- Format: `MOV rd, rs1` or `MOV rd, #immediate`

### Arithmetic Operations

**`ADD`, `SUB`, `MUL` instructions**
- Basic arithmetic with overflow protection (wrapping)
- Support register-register or register-immediate operations
- Format: `ADD rd, rs1, rs2` or `ADD rd, rs1, #immediate`

**`DIV` instruction**
- Integer division with zero-division error checking
- Sets processor flags based on result
- Format: `DIV rd, rs1, rs2`

**`CMP` instruction**
- Compare two values and set flags
- Used for conditional jumps
- Format: `CMP rs1, rs2` or `CMP rs1, #immediate`

### Bitwise Operations

**`AND`, `OR`, `XOR`, `NOT` instructions**
- Bitwise logical operations
- Set flags based on results
- Format: `AND rd, rs1, rs2`, `OR rd, rs1, rs2`, etc.

**`LSL`, `LSR` instructions**
- Logical shift left/right
- Validates shift amounts (must be < 32)
- Format: `LSL rd, rs1, #shift_amount`

### Control Flow

**Jump Instructions**
- `JMP`: Unconditional jump to address
- `JEQ`: Jump if equal (zero flag set)
- `JNE`: Jump if not equal (zero flag clear)
- `JLT`: Jump if less than (negative flag set)
- `JGT`: Jump if greater than (not negative and not zero)

**Function Calls**
- `CALL`: Push return address to stack and jump
- `RET`: Pop return address and jump back
- Maintains call stack for debugging

### Stack Operations

**`PUSH` instruction**
- Push register value onto stack
- Format: `PUSH rd`

**`POP` instruction**
- Pop value from stack into register
- Error checking for stack underflow
- Format: `POP rd`

### I/O System Calls

**`PRINT` instruction**
- Print register value as integer to stdout
- Uses buffered output for performance
- Format: `PRINT rd`

**`PRINTC` instruction**
- Print character or string
- If register value is 0, prints null-terminated string from memory address 512
- Otherwise prints single character
- Format: `PRINTC rd`

**`INPUT` instruction**
- Read input from stdin with multiple modes:
    - Mode 0: Read integer into register
    - Mode 1: Read single character
    - Mode 2: Read string into memory buffer
    - Mode 3: Read floating-point number (stored as bits)
- Input mode specified in rs1 register
- Format: `INPUT rd, rs1` (mode) or `INPUT rd, rs1, rs2` (string mode with base address)

### System Control

**`HALT` instruction**
- Stops VM execution
- Flushes output buffer
- Sets running flag to false

**`NOP` instruction**
- No operation (does nothing)
- Used for padding or debugging

## Error Handling

The runtime provides comprehensive error handling:
- **Division by zero**: Detected and reported with stack trace
- **Memory bounds checking**: Prevents out-of-bounds access
- **Stack underflow**: Detected on POP/RET operations
- **Invalid input**: Handles malformed user input gracefully
- **Shift overflow**: Validates shift amounts for bit operations

## Debugging Features

**`debug_state()`**
- Displays current VM state including:
    - Program counter and instruction count
    - Register values (first 8 registers)
    - Processor flags
    - Stack and call stack sizes

**Stack Traces**
- Automatic stack trace generation on errors
- Shows instruction sequence leading to error
- Includes program counter values for debugging

## Performance Features

- **Buffered I/O**: Uses `BufWriter` for efficient output
- **Inline functions**: Critical path functions marked with `#[inline]`
- **Wrapping arithmetic**: Prevents panic on overflow
- **Efficient instruction decoding**: Bit manipulation for fast instruction parsing

The virtual machine is designed for both educational purposes and
practical assembly language execution, providing a complete virtual
machine environment with robust error handling and debugging capabilities.