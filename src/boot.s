# boot.S

# Disable generation of compressed instructions.
.option norvc

# Define a .data section.
.section .data

# Define a .text.init section.
.section .text.init
# Execution starts here.
.global _start
_start:

	# Disable linker instruction relaxation for the `la` instruction below.
	# This disallows the assembler from assuming that `gp` is already initialized.
	# This causes the value stored in `gp` to be calculated from `pc`.
	# The job of the global pointer is to give the linker the ability to address
	# memory relative to GP instead of as an absolute address.

.option push
.option norelax
	la		gp, _global_pointer
.option pop

	# SATP should be zero, but let's make sure. Each HART has its own
	# SATP register.
	csrw	satp, zero
	# Any hardware threads (hart) that are not bootstrapping
	# need to wait for an IPI
	csrr	t0, mhartid
	bnez	t0, 3f

	# Set all bytes in the BSS section to zero.
	la 		a0, _bss_start
	la		a1, _bss_end
	bgeu	a0, a1, 2f

1:
	sd		zero, (a0)
	addi	a0, a0, 8
	bltu	a0, a1, 1b
2:

	# The stack grows from bottom to top, so we put the stack pointer
	# to the very end of the stack range.
	la		sp, _stack_end
	# Setting `mstatus` register:
	# 0b11 << 11: Machine's previous protection mode is 3 (MPP=3).
	# 1 << 7    : Machine's previous interrupt-enable bit is 1 (MPIE=1).
	# 1 << 3    : Machine's interrupt-enable bit is 1 (MIE=1).
	li		t0, (0b11 << 11) | (1 << 7) | (1 << 3)
	csrw	mstatus, t0
	# Machine's exception program counter (MEPC) is set to `kmain`.

	la		t1, kmain
	csrw	mepc, t1
	# Machine's trap vector base address is set to `asm_trap_vector`.
	la		t2, asm_trap_vector
	csrw	mtvec, t2

	# Setting Machine's interrupt-enable bits (`mie` register):
	# 1 << 3 : Machine's M-mode software interrupt-enable bit is 1 (MSIE=1).
	# 1 << 7 : Machine's timer interrupt-enable bit is 1 (MTIE=1).
	# 1 << 11: Machine's external interrupt-enable bit is 1 (MEIE=1).
	li		t3, (1 << 3) | (1 << 7) | (1 << 11)
	csrw	mie, t3

	# Set the return address to infinitely wait for interrupts.
	la		ra, 4f
	# We use mret here so that the mstatus register is properly updated.

	mret

3:
	# Parked harts go here. We need to set these
	# to only awaken if it receives a software interrupt,
	# which we're going to call the SIPI (Software Intra-Processor Interrupt).
	# We only use these to run user-space programs, although this may
	# change.

4:
	wfi
	j		4b

# In the future our trap vector will go here.

.global asm_trap_vector
# This will be our trap vector when we start
# handling interrupts.
asm_trap_vector:
	mret
