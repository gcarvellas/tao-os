global _start
extern kernel_main

bits 32

; Flags for _large_ p2 aka. PDE page table entries
PDE_PRESENT  equ 1 << 0
PDE_WRITABLE equ 1 << 1
PDE_LARGE    equ 1 << 7

_start:

.setup_stack_pointer:
    mov esp, stack_top

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

    ; Set the p2[1] entry to point to the _second_ 2 MiB frame
	mov eax, (0x20_0000 | PDE_PRESENT | PDE_WRITABLE | PDE_LARGE)
	mov [p2_table + 8], eax

	; point the 0th entry to the first frame
	mov eax, (0x00_0000 | PDE_PRESENT | PDE_WRITABLE | PDE_LARGE)
	mov [p2_table], eax

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

    lgdt [gdt64.pointer]
    jmp gdt64.code:longstart
   
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
stack_bottom:
	resb 4096
stack_top:

section .rodata
gdt64:
	dq 0
.code: equ $ - gdt64
	dq (1 << 43) | (1 << 44) | (1 << 47) | (1 << 53)
.pointer:
	dw $ - gdt64 - 1 ; length of the gdt64 table
	dq gdt64         ; addess of the gdt64 table

section .text
