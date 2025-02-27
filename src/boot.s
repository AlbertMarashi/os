# Disable generation of compressed instructions.
.option norvc

# Define a .data section.
.section .data

# Define a .text.init section.
.section .text.init

# Execution starts here.
.global _start
_start:
    # Any hardware threads (hart) that are not bootstrapping
    # need to wait for an IPI.
    csrr    t0, mhartid
    bnez    t0, secondary_hart  # If we're not hart 0, park the hart

    # Initialize global pointer
.option push
.option norelax
    la      gp, _global_pointer
.option pop

    # Zero out the BSS section.
    la      a0, _bss_start
    la      a1, _bss_end
    bgeu    a0, a1, 2f
1:
    sd      zero, (a0)
    addi    a0, a0, 8
    bltu    a0, a1, 1b
2:
    # Set up the stack pointer
    la      sp, _stack_end
    
    # Set mstatus register
    # - Set MPP (bits 11-12) to 0b11 for Machine mode after mret
    # - Set MPIE (bit 7) to 1 to enable interrupts after mret
    # - Set MIE (bit 3) to 1 to enable interrupts in general
    li      t0, (0b11 << 11) | (1 << 7) | (1 << 3)
    csrw    mstatus, t0
    
    # Set machine's trap vector base address
    la      t2, asm_trap_vector
    csrw    mtvec, t2

    # Call kmain directly
    call    kmain
    
    # If we get here from kmain, jump to end loop
    j       end_loop

# Secondary hart parking
secondary_hart:
    # Simply park secondary harts for now
    wfi
    j       secondary_hart

end_loop:
    # End loop in case kmain returns (should never happen)
    j       end_loop

# Simple trap vector - just return for now
.global asm_trap_vector
asm_trap_vector:
    mret
