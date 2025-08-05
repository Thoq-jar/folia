.text
.start main

main:
    mov r1, #0              ; initialize loop counter to 0
    mov r2, #1000000        ; set loop limit to 1000000

loop:
    cmp r1, r2              ; compare counter with limit
    jeq end                 ; jump to end if counter >= limit

    add r1, r1, #1          ; increment

    print r1                ; print
    mov r3, #10             ; newline
    printc r3

    jmp loop                ; back to loop

end:
    mov r0, 0               ; set exit code to success
    halt                    ; halt program
