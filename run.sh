#!/bin/zsh

FLAGS=(
    -machine virt
    -cpu rv64
    -smp 4
    #-serial mon:stdio
    -nographic
    -m 128M
    -bios none
    -drive format=raw,file=hdd.dsk,id=dr0
    -device virtio-blk-device,drive=dr0
    -device virtio-gpu-device
)

qemu-system-riscv64 $FLAGS -kernel $1