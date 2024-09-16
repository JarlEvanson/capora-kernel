OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)

ENTRY(_start)

PHDRS {
    headers         PT_PHDR  PHDRS              ;
    rodata          PT_LOAD  PHDRS  FLAGS(4)    ;
    text            PT_LOAD         FLAGS(1 | 4);
    data            PT_LOAD         FLAGS(2 | 4);
    dynamic         PT_DYNAMIC                  ;
    boot_request    0x69B2Ba6E                  ;
}

SECTIONS {
    phdrs_start = . + 64; /* Skip the ELF file header. */
    . += SIZEOF_HEADERS;
    phdrs_end = .;

    .rodata : {
        *(.rodata .rodata.*)
    } :rodata

    . = ALIGN(CONSTANT(COMMONPAGESIZE));

    .text : {
        *(.text .text.*)
    } :text

    . = ALIGN(CONSTANT(COMMONPAGESIZE));

    .data : {
        *(.data .data.*)
    } :data

    .limine_requests : {
        KEEP(*(.limine_requests))
    } :data

    .bss : {
        *(.bss .bss.*)
    } :data

    .got : {
        *(.got .got.*)
    } :data

    .dynamic : {
        *(.dynamic .dynamic.*)
    } :dynamic

    .bootloader_request : {
        KEEP(*(.bootloader_request))
    } :boot_request
}
