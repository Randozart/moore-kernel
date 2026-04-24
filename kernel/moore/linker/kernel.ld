/* kernel.ld - Linker script for Moore Kernel on KV260
 *     Copyright (C) 2026 Randy Smits-Schreuder Goedheijt
 *
 * Target: ARM Cortex-A53 @ 1.2 GHz, Zynq UltraScale+ MPSoC
 */

OUTPUT_FORMAT("elf64-littleaarch64")
OUTPUT_ARCH(aarch64)
ENTRY(moore_main)

/* DDR4 Memory - 2GB starting at 0x0000_0000 */
MEMORY
{
    DDR4 (rw) : ORIGIN = 0x00000000, LENGTH = 0x80000000
    OCM (rw)  : ORIGIN = 0xFFF00000, LENGTH = 0x40000
    SRAM (rw) : ORIGIN = 0xFFE00000, LENGTH = 0x20000
}

/* Kernel code and data goes into DDR4 */
SECTIONS
{
    .text 0x00000000 : {
        *(.text.boot)
        *(.text*)
        *(.rodata*)
    } > DDR4

    .data : {
        *(.data*)
    } > DDR4

    .bss : {
        *(.bss*)
        *(COMMON)
    } > DDR4

    .stack (NOLOAD) : {
        . = ALIGN(16);
        _stack_start = .;
        . += 0x10000;
        _stack_end = .;
    } > DDR4

    /* Exception vectors - placed at start of DDR4 */
    .vectors 0x00000000 : {
        *(.vectors)
    } > DDR4

    /* Device tree blob - if we include one */
    .fdt : {
        *(.fdt)
    } > DDR4
}

_stack_start = 0x000FFFF0;
_heap_start = _stack_end;
_heap_end = 0x7FFFFFFF;