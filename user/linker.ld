/* User-space ELF linker script (load address 0x400000). */
ENTRY(_start)

SECTIONS
{
    . = 0x400000;

    .text : ALIGN(4K) {
        KEEP(*(.text._start))
        *(.text .text.*)
    }

    .rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
    }

    .data : ALIGN(4K) {
        *(.data .data.*)
    }

    .bss : ALIGN(4K) {
        *(.bss .bss.*)
        *(COMMON)
    }
}
