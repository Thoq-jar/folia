# Folia Virtual Machine - Call Reference

| Instruction | Opcode             | Parameters                                                         | Description                                                                                                                                 | Error Conditions                          |
|-------------|--------------------|--------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------|-------------------------------------------|
| **PRINT**   | `PRINT rd`         | `rd`: Register containing integer                                  | Print register value as integer to stdout                                                                                                   | None                                      |
| **PRINTC**  | `PRINTC rd`        | `rd`: Register with character code or 0                            | Print character or null-terminated string from memory[512] if rd=0                                                                          | None                                      |
| **INPUT**   | `INPUT rd, rs1`    | `rd`: Destination register<br>`rs1`: Input mode (0-3)              | Read from stdin based on mode:<br>• 0: Integer<br>• 1: Single character<br>• 2: String (requires rs2 for base addr)<br>• 3: Float (as bits) | Invalid mode, parse errors, memory bounds |
| **HALT**    | `HALT`             | None                                                               | Terminate program execution and flush output                                                                                                | None                                      |
| **LOAD**    | `LOAD rd, [rs1]`   | `rd`: Destination register<br>`rs1`: Address register or immediate | Load byte from memory address into register                                                                                                 | Memory bounds violation                   |
| **STORE**   | `STORE rd, [addr]` | `rd`: Source register<br>`addr`: Memory address                    | Store register value to memory address                                                                                                      | Memory bounds violation                   |
| **PUSH**    | `PUSH rd`          | `rd`: Register to push                                             | Push register value onto runtime stack                                                                                                      | Stack overflow (implicit)                 |
| **POP**     | `POP rd`           | `rd`: Destination register                                         | Pop value from stack into register                                                                                                          | Stack underflow                           |
| **CALL**    | `CALL addr`        | `addr`: Function address                                           | Push return address and jump to function                                                                                                    | Stack overflow (implicit)                 |
| **RET**     | `RET`              | None                                                               | Pop return address and return to caller                                                                                                     | Stack underflow                           |

## Input Mode Details (INPUT instruction)

| Mode      | Type                 | rs1 Value | Additional Parameters      | Behavior                                            |
|-----------|----------------------|-----------|----------------------------|-----------------------------------------------------|
| Integer   | `INPUT rd, rs1`      | 0         | None                       | Parse stdin as 32-bit signed integer                |
| Character | `INPUT rd, rs1`      | 1         | None                       | Read first character from stdin as ASCII value      |
| String    | `INPUT rd, rs1, rs2` | 2         | `rs2`: Base memory address | Store string at memory address, return length in rd |
| Float     | `INPUT rd, rs1`      | 3         | None                       | Parse stdin as f32, store as bit representation     |

## Error Handling

All system calls return runtime errors with:
- **Stack traces** showing instruction sequence
- **Program counter** location of error
- **Detailed error messages** describing the failure condition

## Memory Layout

| Address Range | Purpose       | System Call Access               |
|---------------|---------------|----------------------------------|
| 0-511         | Reserved      | None                             |
| 512+          | Data section  | LOAD/STORE, PRINTC string output |
| Stack         | Runtime stack | PUSH/POP/CALL/RET                |
