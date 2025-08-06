do_add:
    call get_two_numbers
    add r1 r2 r3
    call print_result
    jmp main_loop

do_subtract:
    call get_two_numbers
    sub r1 r2 r3
    call print_result
    jmp main_loop

do_multiply:
    call get_two_numbers
    mul r1 r2 r3
    call print_result
    jmp main_loop

do_divide:
    call get_two_numbers
    cmp r3 #0
    jeq div_by_zero
    div r1 r2 r3
    call print_result
    jmp main_loop

div_by_zero:
    mov r1 div_error_msg
    call print_string
    jmp main_loop
