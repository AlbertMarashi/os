## Install Dependencies
- install qemu
- rustup default nightly
- rustup target add riscv64gc-unknown-none-elf
- cargo install cargo-binutils

## Create hdd.dsk

### MacOS
mkfile -n 1m hdd.dsk

### Linux
fallocate -l 1M hdd.dsk