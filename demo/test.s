.start main

.data
    hello_msg: .string "Hello, World!"

.text
    main:
        LEA r0, [hello_msg]  ; load message
        MOV r2, #10          ; \n
        PRC r0
        PRC r2

        HLT
