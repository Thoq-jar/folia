.start _start

.data
    welcome_msg: .string "Simple Calculator\n"
    menu_msg: .string "1. Add\n2. Subtract\n3. Multiply\n4. Divide\n5. Exit\nChoice: "
    num1_prompt: .string "Enter first number: "
    num2_prompt: .string "Enter second number: "
    result_msg: .string "Result: "
    div_error_msg: .string "Error: Division by zero!\n"
    invalid_choice_msg: .string "Invalid choice!\n"
    newline: .string "\n"

.text
_start:
    mov r1 welcome_msg
    call print_string

main_loop:
    mov r1 menu_msg
    call print_string

    mov r0 #0
    input r1 r0

    cmp r1 #1
    jeq do_add

    cmp r1 #2
    jeq do_subtract

    cmp r1 #3
    jeq do_multiply

    cmp r1 #4
    jeq do_divide

    cmp r1 #5
    jeq exit_program

    mov r1 invalid_choice_msg
    call print_string
    jmp main_loop

get_two_numbers:
    mov r1 num1_prompt
    call print_string
    mov r0 #0
    input r2 r0

    mov r1 num2_prompt
    call print_string
    mov r0 #0
    input r3 r0
    ret

print_result:
    push r1
    mov r1 result_msg
    call print_string
    pop r1
    print r1
    mov r1 newline
    call print_string
    ret

print_string:
    push r1
    push r3
print_loop:
    load r3 [r1]
    cmp r3 #0
    jeq end_print
    printc r3
    add r1 r1 #1
    jmp print_loop
end_print:
    pop r3
    pop r1
    ret

exit_program:
    halt
