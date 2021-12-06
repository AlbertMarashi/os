ENTRY(_start)

SECTIONS {
    . = 0x80000000;

    .text : {
        *(.text .text.*)
        . = ALIGN(0x2000);
    }

    _global_pointer = .;

    .rodata : {
        *(.rodata .rodata.*)
        . = ALIGN(0x2000);
    }

    .data : {
        *(.data .data.*)
        . = ALIGN(0x2000);
    }

    .bss : {
        _bss_start = .;
        *(.bss .bss.*)
        *(.COMMON)
        . = ALIGN(0x2000);
        _bss_end = .;
    }

    . += 0x10000;
    _stack = .;
}
