KERNEL_VMA = 2M;

ENTRY(_start)

SECTIONS
{
    . = KERNEL_VMA;

    KERNEL_START = .;

    .multiboot :
    {
        . = ALIGN(8);
        KEEP(*(.multiboot))
    }

    .text BLOCK(4K) : ALIGN(4096) 
    {
        *(.text)
    }

    .rodata BLOCK(4K) : ALIGN(4096)
    {
        *(.rodata)
    }

   .data BLOCK(4K): ALIGN(4096)
   {
        *(.data)
   }

   .bss BLOCK(4K): ALIGN(4096)
   {
       *(COMMON)
       *(.bss)
   }

   	/* Without this kernel end is not incremented to avoid colliding with bss */
	.phony : {

	}
    KERNEL_END = .;

   /DISCARD/ :
   {
        *(.comment)
   }
}
