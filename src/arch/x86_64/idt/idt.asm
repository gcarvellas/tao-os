extern int20h_handler
extern no_interrupt_handler

global int20h
global no_interrupt

%include "./src/arch/x86_64/macros.asm"

no_interrupt:
    pushaq
    call no_interrupt_handler
    popaq
    iretq

int20h:
    pushaq
    call int20h_handler
    popaq
    iretq
