FILES = ./build/kernel.asm.o ./build/kernel.o
INCLUDES = -I./src
CFLAGS = -static -g -ffreestanding -falign-jumps -falign-functions -falign-labels -falign-loops \
		 -fstrength-reduce -fomit-frame-pointer -finline-functions -Wno-unused-function \
		 -fno-builtin -Werror -Wno-unused-label -Wno-cpp -Wno-unused-parameter -nostdlib \
		 -nostartfiles -nodefaultlibs -Wall -O0 -Iinc -fno-pie -no-pie
RUST_FLAGS = -C link-arg=./linker.ld -C relocation-model=static \
			 -C prefer-dynamic=false --emit=obj

# TODO there's hardcoded flags everywhere for debugging

all: ./build/boot/boot.bin ./build/kernel.bin
	dd if=./build/boot/boot.bin >> ./build/os.bin
	dd if=./build/kernel.bin >> ./build/os.bin

	# TODO I think this is for alignment purposes... not sure 
	dd if=/dev/zero bs=1048576 count=16 >> ./build/os.bin

./build/boot/boot.bin:
	mkdir --parents ./build/boot
	nasm -f bin ./src/boot/boot.asm -o ./build/boot/boot.bin

./build/kernel.bin: $(FILES)
	ld -g -relocatable $(FILES) -o ./build/kernelfull.o
	gcc -T ./linker.ld -o ./build/kernel.bin -ffreestanding -O0 -nostdlib ./build/kernelfull.o

./build/kernel.asm.o:
	nasm -f elf64 -g ./src/kernel.asm -o ./build/kernel.asm.o

./build/kernel.o:
	rustc $(RUST_FLAGS) --target x86_64-unknown-none -o ./build/kernel.o ./src/kernel.rs

clean:
	rm -rf build
