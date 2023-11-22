extern no_interrupt_handler
extern pushaq
extern popaq
extern interrupt_handler

global idt_load
global no_interrupt
global interrupt_pointer_table

idt_load:
    push rbp
    mov rbp, rsp

    mov rbx, [rbp+8]
    lidt [rbx]
    pop rbp
    ret

no_interrupt:
    pushaq
    call no_interrupt_handler
    popaq
    iretq

%macro interrupt 1
    global int%1
    int%1:
        ; Cannot call the pushaq macro in another macro
        push rax
        push rcx
        push rdx
        push rbx
        push rbp
        push rsi
        push rdi
        push r8
        push r9
        push r10
        push r11
        push r12
        push r13
        push r14
        push r15

        push rsp
        push dword %1
        call interrupt_handler
        add rsp, 8

        ; Cannot call the popaq macro in another macro
        pop rax
        pop rcx
        pop rdx
        pop rbx
        pop rbp
        pop rsi
        pop rdi
        pop r8
        pop r9
        pop r10
        pop r11
        pop r12
        pop r13
        pop r14
        pop r15

        iretq
%endmacro

%assign i 0
%rep 512
    interrupt i
%assign i i+1
%endrep

section .data

tmp_res: dd 0

%macro interrupt_array_entry 1
    dd int%1
%endmacro

interrupt_pointer_table:
%assign i 0
%rep 512
    interrupt_array_entry i
%assign i i+1
%endrep
