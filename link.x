MEMORY {
    /* 0x60300 here matches the bootloader's initial jump to 768 */
    FLASH : ORIGIN = 0x60060300, LENGTH = 256K
    RAM   : ORIGIN = 0x61000000, LENGTH = 2048K
}

_data_lma = LOADADDR(.data);
_data_size = SIZEOF(.data);
_bss_size = SIZEOF(.bss);
_ram_top = ORIGIN(RAM) + LENGTH(RAM);

ENTRY(_start)

SECTIONS {

    /* Mash read-only sections together for easy extraction with objcopy */
    .firmware : {
        *(.text._start)  /* initialization code MUST come first */
        *(.text*)
        . = ALIGN(16);   /* 16 to make it look pretty in hexdump -C */
        *(.rodata)
        . = ALIGN(16);
    } > FLASH

    /* This gets its own section to make the LMA & VMA addressing clear */
    .data : {
        _data_vma = .;
        KEEP(*(.data*))
        . = ALIGN(16);
    } > RAM AT > FLASH

    .bss (NOLOAD) : {
        _bss_vma = .;
        *(.bss)
        . = ALIGN(16);
    } > RAM

    /* Drop these for smaller file size and better reproducibility. These
     * sections have stack unwinding metadata, gdb stuff, etc.
     */
    /DISCARD/ : {
        *(.eh_frame*)
        *(.comment*)
        *(.riscv.attributes*)
        *(.debug*)
    }
}
