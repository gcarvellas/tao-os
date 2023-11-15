SRCDIR := src
BUILDDIR := build
RUST_FLAGS = +nightly -C link-arg=./linker.ld -C relocation-model=static \
			 -C prefer-dynamic=false --emit=obj

ASMSOURCES := $(shell find $(SRCDIR) -name '*.asm')
OBJFILES := $(patsubst $(SRCDIR)/%.asm, $(BUILDDIR)/%.o, $(ASMSOURCES))

LINKER_SCRIPT := linker.ld

RUST_KERNEL_OBJ  := $(BUILDDIR)/kernel.o
KERNEL_BIN  := $(BUILDDIR)/kernel.bin

all: $(KERNEL_BIN)
	mkdir -p $(BUILDDIR)/isofiles/boot/grub
	cp $(KERNEL_BIN) $(BUILDDIR)/isofiles/boot/kernel.bin
	cp ./grub.cfg $(BUILDDIR)/isofiles/boot/grub/
	grub-mkrescue -o $(BUILDDIR)/tao-os.iso $(BUILDDIR)/isofiles
	grub-file --is-x86-multiboot2 $(BUILDDIR)/tao-os.iso

$(BUILDDIR)/%.o: $(SRCDIR)/%.asm
	mkdir -p $(dir $@)
	nasm -f elf64 $< -o $@

$(KERNEL_BIN): $(OBJFILES) $(RUST_KERNEL_OBJ)
	ld -T $(LINKER_SCRIPT) $^ -o $@

$(RUST_KERNEL_OBJ):
	~/.cargo/bin/rustc $(RUST_FLAGS) --target x86_64-unknown-none -o ./build/kernel.o ./src/kernel.rs

clean:
	rm -rf build
