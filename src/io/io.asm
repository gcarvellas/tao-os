global _insb
global _insw
global _outb
global _outw
_insb:
    ; Setup stack
    push rbp
    mov rbp, rsp

    ; Get port
    xor rax, rax ; Set to 0
    mov rdx, [rbp+8]

    in al, dx ;al is part of the eax register, which is the return value

    ; Finish stack
    pop rbp
    ret

_insw:
    push rbp
    mov rbp, rsp

    xor rax, rax
    mov rdx, [rbp+8]

    in ax, dx

    pop rbp
    ret

_outb:
    push rbp
    mov rbp, rsp

    mov rax, [rbp+12]
    mov rdx, [rbp+8]
    out dx, al

    pop rbp
    ret

_outw:
    push rbp
    mov rbp, rsp

    mov rax, [rbp+12]
    mov rdx, [rbp+8]
    out dx, ax

    pop rbp
    ret
