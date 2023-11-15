SRCDIR := src
BUILDDIR := build
RUST_FLAGS = +nightly -C link-arg=./linker.ld -C relocation-model=static \
			 -C prefer-dynamic=false --emit=obj

ASMSOURCES := $(shell find $(SRCDIR) -name '*.asm')
OBJFILES := $(patsubst $(SRCDIR)/%.asm, $(BUILDDIR)/%.o, $(ASMSOURCES))

LINKER_SCRIPT := linker.ld

RUST_KERNEL_OBJ  := $(BUILDDIR)/kernel.o
KERNEL_ELF  := $(BUILDDIR)/kernel.elf

all: $(KERNEL_ELF)
	mkdir -p $(BUILDDIR)/isofiles/boot/grub
	cp $(KERNEL_ELF) $(BUILDDIR)/isofiles/boot/kernel.elf
	cp ./grub.cfg $(BUILDDIR)/isofiles/boot/grub/
	grub-mkrescue -o $(BUILDDIR)/tao-os.iso $(BUILDDIR)/isofiles
	grub-file --is-x86-multiboot2 $(BUILDDIR)/tao-os.iso

$(BUILDDIR)/%.o: $(SRCDIR)/%.asm
	mkdir -p $(dir $@)
	nasm -f elf64 $< -o $@

$(KERNEL_ELF): $(OBJFILES) $(RUST_KERNEL_OBJ)
	ld -m elf_x86_64 -nostdlib -T $(LINKER_SCRIPT) $^ -o $@

$(RUST_KERNEL_OBJ):
	~/.cargo/bin/rustc $(RUST_FLAGS) --target x86_64-unknown-none -o $(BUILDDIR)/kernel.o ./src/kernel.rs

clean:
	rm -rf build
