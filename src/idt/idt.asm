extern int20h_handler
extern no_interrupt_handler

global int20h
global no_interrupt

%include "./src/arch/macros.asm"

no_interrupt:
    cli
    pushaq
    call no_interrupt_handler
    popaq
    sti
    iretq

int20h:
    cli
    pushaq
    call int20h_handler
    popaq
    sti
    iretq
