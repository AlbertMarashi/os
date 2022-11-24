#!/bin/zsh

FLAGS=(
    -machine virt
    -cpu rv64
    -smp 4
    -serial mon:stdio
    -nographic
    -m 128M
    -bios none
    -drive if=none,format=raw,file=hdd.dsk,id=disk_1
    -device virtio-blk-device,drive=disk_1
    -device virtio-gpu-device
    -device virtio-rng-device
    -device virtio-net-device
    -device virtio-tablet-device
    -device virtio-keyboard-device
)

qemu-system-riscv64 $FLAGS -kernel $1