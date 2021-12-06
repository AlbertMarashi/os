## Install Dependencies
- install qemu
  - QEMU (for emulating): [Download QEMU](https://www.qemu.org/download/)
- rustup default nightly
- rustup target add riscv64gc-unknown-none-elf
- cargo install cargo-binutils

## Create hdd.dsk

### MacOS
mkfile -n 1m hdd.dsk

### Linux
fallocate -l 1M hdd.dsk

---
Based off of Stephen's Rust RiscV Blog OS