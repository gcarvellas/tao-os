KERNEL_VMA = 2M;

ENTRY(_start)

MAGIC_NUMBER = 0xe85250d6;

SECTIONS
{
    . = KERNEL_VMA;

    KERNEL_START = .;

    .boot :
    {
        header_start = .;

        /* Magic number */
        LONG(MAGIC_NUMBER)
        /* Architecture 0: Protected mode i386 */
        LONG(0)
        /* Header length */
        LONG(header_end - header_start)
        /* Checksum */
        LONG(0x100000000 - (MAGIC_NUMBER + (header_end - header_start)))
        /* Required end tag */
        /* Type */
        SHORT(0)
        /* Flags */
        SHORT(0)
        /* Size */
        LONG(8)

        header_end = .;
  }

  .text BLOCK(4K) : ALIGN(4096) 
  {
          KEEP(*(.text.start))
          *(.text)
          *(.text*)
  }

.rodata BLOCK(4K) : ALIGN(4096)
    {
        *(.rodata)
        *(.rodata*)
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

   .debug BLOCK(4K) : ALIGN(4096)
   {
        *(.debug)
        *(.debug*)
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
