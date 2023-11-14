KERNEL_VMA = 1M;

ENTRY(_start)
OUTPUT_FORMAT(binary)

MEMORY
{
    PAGES (rw) : ORIGIN = 0x10000, LENGTH = 2M
}

SECTIONS
{
    . = 3M;

    .text : ALIGN(4096)
    {
        _code = .;
        *(.text)
    }

    .asm : ALIGN(4096)
    {
        _asm = .;
        *(.asm)
    }

    .rodata : ALIGN(4096)
    {
        _rodata = .;
        *(.rodata)
    }

   .data : ALIGN(4096)
   {
        _data = .;
        *(.data)
   }

   .bss : ALIGN(4096)
   {
       _bss = .;
       *(.bss)

       /*
        * You usually need to include generated COMMON symbols
        * under kernel BSS section or use gcc's -fno-common
        */

        *(COMMON)
   }

   _end = .;

   /DISCARD/ :
   {
        *(.comment)
   }
}
