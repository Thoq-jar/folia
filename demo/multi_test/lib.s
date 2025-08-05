loop:
    cmp r3, r4              ; compare counter with limit
    jgt end                 ; jump to end if counter > limit

                            ; calculate next fibonacci number
    add r6, r1, r2          ; r6 = r1 + r2

                            ; print the result
    print r6
    printc r5               ; newline

    mov r1, r2              ; r1 = previous r2
    mov r2, r6              ; r2 = new fibonacci number

    add r3, r3, #1          ; increment counter
    jmp loop                ; back to loop

end:
    halt                    ; halt program
