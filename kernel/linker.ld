ENTRY(_start)

SECTIONS
{
    . = 0x100000;

    __kernel_start = .;

    .text : ALIGN(4K) {
        __kernel_text_start = .;
        KEEP(*(.text._start))
        *(.text .text.*)
        __kernel_text_end = .;
    }

    .rodata : ALIGN(4K) {
        __kernel_rodata_start = .;
        *(.rodata .rodata.*)
        __kernel_rodata_end = .;
    }

    .data : ALIGN(4K) {
        __kernel_data_start = .;
        *(.data .data.*)
        __kernel_data_end = .;
    }

    .bss : ALIGN(4K) {
        __kernel_bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __kernel_bss_end = .;
    }

    __kernel_end = .;

    /DISCARD/ : {
        *(.comment)
    }
}
