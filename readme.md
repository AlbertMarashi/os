## Install Dependencies
- install qemu
  - QEMU (for emulating): [Download QEMU](https://www.qemu.org/download/)
- rustup default nightly
- rustup target add riscv64gc-unknown-none-elf
- cargo install cargo-binutils

## How it works
`.cargo` contains information for cargo on how to build and run the OS on the emulator with `cargo run`

It runs `run.sh` which runs the qemu emulator with basic configurations.

The binary is built using the `riscv64gc-unknown-none-elf` compilation target.

in `src/linker/virt.linker` you will find the linker script used to set up the binary executable.
it is well documented and should not need much if any configuation.

the entry point of our program is _start in (found in `src/linker/virt.linker`). This links to a
_start symbol found in `boot.s` which contains some assembly code to initialize everything.

`boot.s` is imported by `main.rs`, and eventually jumps to `kmain` which is the entry point of our OS.

## Create hdd.dsk
We need to create a hard drive disk for the qemu emulator.
### MacOS
mkfile -n 32m hdd.dsk

### Linux
fallocate -l 1M hdd.dsk


> Based off of Stephen's Rust RiscV Blog OS