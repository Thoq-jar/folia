.text
.start main

main:
    mov r1, #0              ; first fibonacci number (F0)
    mov r2, #1              ; second fibonacci number (F1)
    mov r3, #0              ; counter
    mov r4, #42             ; limit (calculate F0 to F42)

    print r1
    mov r5, #10             ; newline
    printc r5

    print r2
    printc r5

    mov r3, #2              ; start counter at 2 (already printed F0 and F1)

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
