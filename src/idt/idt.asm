extern no_interrupt_handler
extern int20h_handler

global idt_load
global int20h
global no_interrupt

%include "./src/utils/macros.asm"

idt_load:
    push rbp
    mov rbp, rsp

    mov rbx, [rbp+8]
    lidt [rbx]
    pop rbp
    ret

int20h:
    cli
    pushaq
    call int20h_handler
    popaq
    sti
    iretq

no_interrupt:
    cli
    pushaq
    call no_interrupt_handler
    popaq
    sti
    iretq
