extern int20h_handler
extern no_interrupt_handler

global int20h
global no_interrupt

%include "./src/utils/macros.asm"

no_interrupt:
    pushaq
    call no_interrupt_handler
    popaq
    add rsp, 2*8
    iretq

int20h:
    pushaq
    call int20h_handler
    popaq
    add rsp, 2*8
    iretq
