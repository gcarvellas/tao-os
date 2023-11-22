SRCDIR := src
BUILDDIR := build
LINKER_SCRIPT := linker.ld

RUST_FLAGS = +nightly
ASM_FLAGS = -g -f elf64
LINKER_FLAGS = -g -m elf_x86_64 -nostdlib

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
	nasm $(ASM_FLAGS) $< -o $@

$(KERNEL_ELF): $(OBJFILES) $(RUST_KERNEL_OBJ)
	ld $(LINKER_FLAGS) -T $(LINKER_SCRIPT) $^ -o $@

$(RUST_KERNEL_OBJ):
	cargo $(RUST_FLAGS) build --target x86_64-unknown-none
	cp target/x86_64-unknown-none/debug/libkernel.a $(BUILDDIR)/kernel.o

docker:
	docker run -t -v .:/mnt tao-os-builder bash -c 'cd /mnt && make all'
 
clean:
	rm -rf build
	cargo clean
