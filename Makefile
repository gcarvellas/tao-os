SRCDIR := src
BUILDDIR := build
LINKER_SCRIPT := linker.ld
RUST_FLAGS = +nightly -C link-arg=$(LINKER_SCRIPT) -C relocation-model=static \
			 -C prefer-dynamic=false --emit=obj

ASMSOURCES := $(shell find $(SRCDIR) -name '*.asm')
OBJFILES := $(patsubst $(SRCDIR)/%.asm, $(BUILDDIR)/%.o, $(ASMSOURCES))

RUST_KERNEL_OBJ  := $(BUILDDIR)/kernel.o
KERNEL_ELF  := $(BUILDDIR)/kernel.elf

ISODIR := $(BUILDDIR)/isofiles

all: $(KERNEL_ELF)
	mkdir -p $(ISODIR)/boot/grub
	cp $(KERNEL_ELF) $(ISODIR)/boot/kernel.elf
	cp ./grub.cfg $(ISODIR)/boot/grub/
	grub-mkrescue -o $(BUILDDIR)/tao-os.iso $(ISODIR)

$(BUILDDIR)/%.o: $(SRCDIR)/%.asm
	mkdir -p $(dir $@)
	nasm -f elf64 $< -o $@

$(KERNEL_ELF): $(OBJFILES) $(RUST_KERNEL_OBJ)
	ld -m elf_x86_64 -nostdlib -T $(LINKER_SCRIPT) $^ -o $@

$(RUST_KERNEL_OBJ):
	rustc $(RUST_FLAGS) --target x86_64-unknown-none -o $(BUILDDIR)/kernel.o ./src/kernel.rs

docker:
	docker run -t -v .:/mnt tao-os-builder bash -c 'cd /mnt && make clean && make all'

clean:
	rm -rf build
