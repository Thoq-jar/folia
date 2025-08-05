.start _start

.data
    hello_msg: .string "Hello, World!\n"
    prompt_msg: .string "Enter a number: "
    result_msg: .string "You entered: "
    newline: .string "\n"

.text
_start:
    MOV r1, hello_msg           ; print hello
    CALL print_string

    MOV r1, prompt_msg          ; prompt user
    CALL print_string

    INPUT r2                    ; get user input

    MOV r1, result_msg          ; result message
    CALL print_string

    PRINT r2                    ; print number user entered

    MOV r1, newline             ; newline
    CALL print_string

    HALT                        ; stop

print_string:
    PUSH r1                     ; save r1
    PUSH r3                     ; save r3 (used for character)
loop:
    LOAD r3, [r1]               ; load character from memory
    CMP r3, #0                  ; check for null terminator
    JEQ end_print               ; if null, exit
    PRINTC r3                   ; print character
    ADD r1, r1, #1              ; move to next character
    JMP loop                    ; repeat
end_print:
    POP r3
    POP r1
    RET
