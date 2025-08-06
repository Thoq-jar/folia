.start main

.text
_start:
    mov r1, #0              ; first fibonacci number (F0)
    mov r2, #1              ; second fibonacci number (F1)
    mov r3, #0              ; counter
    mov r4, #46             ; limit (calculate F0 to F42)

    prt r1
    mov r5, #10             ; newline
    prc r5

    prt r2
    prc r5

    mov r3, #2              ; start counter at 2 (already printed F0 and F1)

loop:
    cmp r3, r4              ; compare counter with limit
    jgt end                 ; jump to end if counter > limit

                            ; calculate next fibonacci number
    add r6, r1, r2          ; r6 = r1 + r2

    prt r6                  ; print the result
    prc r5                  ; newline

    mov r1, r2              ; r1 = previous r2
    mov r2, r6              ; r2 = new fibonacci number

    add r3, r3, #1          ; increment counter
    jmp loop                ; back to loop

end:
    hlt                     ; halt program
