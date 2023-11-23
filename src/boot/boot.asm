; https://wiki.osdev.org/Setting_Up_Long_Mode

global _start
extern kernel_main

bits 32

%macro SETUP_P2_ENTRY 3 ; Macro with 3 parameters: p2_table, index, and frame_address
    mov eax, (%3 | PDE_PRESENT | PDE_WRITABLE | PDE_LARGE)
    mov [ %1 + %2 * 8], eax
%endmacro

; Flags for _large_ p2 aka. PDE page table entries
PDE_PRESENT  equ 1 << 0
PDE_WRITABLE equ 1 << 1
PDE_LARGE    equ 1 << 7

; GDT Access bits
PRESENT        equ 1 << 7
NOT_SYS        equ 1 << 4
EXEC           equ 1 << 3
DC             equ 1 << 2
RW             equ 1 << 1
ACCESSED       equ 1 << 0
 
; GDT Flags bits
GRAN_4K       equ 1 << 7
SZ_32         equ 1 << 6
LONG_MODE     equ 1 << 5

_start:

.setup_stack_pointer:
    mov ebp, 0x00200000 ; base pointer
    mov esp, ebp ; stack pointer

.setup_paging:
    ; Disable paging
    mov eax, cr0
    and eax, ~(1 << 31)
    mov cr0, eax

    ; Enable Physical Address Extension
    mov eax, cr4
    or eax, (1 << 5)
    mov cr4, eax

    ; Set cr3 register
    mov eax, p4_table
    mov cr3, eax
    
    ; Each entry is 2MiB
    %assign i 0
    %rep 10 ;  Num of 2MiB pages
        SETUP_P2_ENTRY p2_table, i, 0x20_0000 * i
    %assign i i+1
    %endrep

	; Set the 0th entry of p3 to point to our p2 table
	mov eax, p2_table ; load the address of the p2 table
	or eax, (PDE_PRESENT | PDE_WRITABLE)
	mov [p3_table], eax

	; Set the 0th entry of p4 to point to our p3 table
	mov eax, p3_table
	or eax, (PDE_PRESENT | PDE_WRITABLE)
	mov [p4_table], eax

	; Set EFER.LME to 1 to enable the long mode
	mov ecx, 0xC0000080
	rdmsr
	or eax, 1 << 8
	wrmsr

	; enable paging
	mov eax, cr0
	or eax, 1 << 31
	mov cr0, eax

    lgdt [GDT.Pointer]
    jmp GDT.Code:longstart
   
section .text
bits 64
longstart:
	call kernel_main

    hlt

    
section .bss
align 4096
p4_table:
    resb 4096
p3_table:
    resb 4096
p2_table:
    resb 4096

section .rodata
GDT:
    .Null: equ $ - GDT
        dq 0
    .Code: equ $ - GDT
        dd 0xFFFF                                   ; Limit & Base (low, bits 0-15)
        db 0                                        ; Base (mid, bits 16-23)
        db PRESENT | NOT_SYS | EXEC | RW            ; Access
        db GRAN_4K | LONG_MODE | 0xF                ; Flags & Limit (high, bits 16-19)
        db 0                                        ; Base (high, bits 24-31)
    .Data: equ $ - GDT
        dd 0xFFFF                                   ; Limit & Base (low, bits 0-15)
        db 0                                        ; Base (mid, bits 16-23)
        db PRESENT | NOT_SYS | RW                   ; Access
        db GRAN_4K | SZ_32 | 0xF                    ; Flags & Limit (high, bits 16-19)
        db 0                                        ; Base (high, bits 24-31)
    .TSS: equ $ - GDT
        dd 0x00000068
        dd 0x00CF8900
    .Pointer:
        dw $ - GDT - 1
        dq GDT

section .text
