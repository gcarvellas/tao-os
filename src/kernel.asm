[BITS 64]

; We cannot put this in the .asm section. This needs to be in the text section

global _start
extern kernel_main

CODE_SEG equ 0x08
DATA_SEG equ 0x10

_start:
    mov ax, DATA_SEG
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov rbp, 0x00200000 ; base pointer
    mov rsp, rbp ; stack pointer
    
    ; Need to enable a20 line. See osdev for more info. This is the fast way to setup a20 line
    in al, 0x92
    or al, 2
    out 0x92, al

    ; Remap the master PIC (programmable interrupt controller)
    mov al, 00010001b ; b4=1: Init, b3=0: Edge, b1=0: Cascade, b0=1: Need 4th init setup
    out 0x20, al ; Tell master

    mov al, 0x20 ; Master IRQ0 should be on INT 0x20 (JUst after intel exceptions)
    out 0x21, al

    mov al, 00000001b ; b4=0: FNM; b3-2=00: Master/Slave set by hardware; b1=0: Not AEOI; b0=1: x86 mode
    out 0x21, al

    ; End remap of the master PIC

	call kernel_main
    jmp $

kernel_registers:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov gs, ax
    mov fs, ax
    ret

times 4096-($ - $$) db 0
