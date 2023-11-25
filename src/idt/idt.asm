extern int20h_handler
extern no_interrupt_handler

global int20h
global no_interrupt

%include "./src/utils/macros.asm"

no_interrupt:
    cli
    pushaq
    mov rdi, rsp
    call no_interrupt_handler
    popaq
    add rsp, 24
    sti
    iretq

int20h:
    cli
    pushaq
    mov rdi, rsp
    call int20h_handler
    popaq
    add rsp, 24
    sti
    iretq
