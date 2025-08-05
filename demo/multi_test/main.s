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
