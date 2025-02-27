#!/bin/zsh

FLAGS=(
    -machine virt
    -cpu rv64
    -smp 4
    -nographic          # No GUI
    -serial mon:stdio   # Combine monitor and serial output on stdio
    -m 512M
    -bios none
    -drive format=raw,file=hdd.dsk,id=dr0,if=none
    -device virtio-blk-device,drive=dr0
    -global virtio-mmio.force-legacy=false
    -device virtio-gpu
)

# FLAGS=(
#     -machine virt
#     -cpu rv64
#     -smp 4
#     -serial mon:stdio
#     -nographic
#     -m 128M
#     -bios none
#     -drive format=raw,file=hdd.dsk,id=dr0,if=none
#     -device virtio-blk-device,drive=dr0
#     # -device virtio-gpu-device
#     -global virtio-mmio.force-legacy=false
# )


FILE=hdd.dsk

# Check if kernel file was provided
if [ -z "$1" ]; then
    echo "Error: No kernel file specified"
    echo "Usage: ./run.sh path/to/kernel.elf"
    exit 1
fi

qemu-system-riscv64 $FLAGS -kernel $1