qemu-system-x86_64 -device isa-debug-exit,iobase=0xf4,iosize=0x01 -drive file=build/tao-os.iso,format=raw,index=0 -serial stdio -display none -drive file=fat16.img,if=ide,format=raw,index=1

status=$?

# Check the status code and map it accordingly
if [ "$status" -eq 33 ]; then
    exit 0
elif [ "$status" -eq 35 ]; then
    exit 1
else
    echo Unknown status code from qemu: $status
    exit $status
fi
