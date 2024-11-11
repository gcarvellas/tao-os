SRCDIR := src
BUILDDIR := build
LINKER_SCRIPT := linker.ld

DEBUG ?= 0
TESTS ?= 0

RUST_FLAGS = +nightly
ASM_FLAGS = -f elf64
LINKER_FLAGS = -m elf_x86_64 -nostdlib
CARGO_BUILD_MODE :=
TARGET_DIR := 

ifeq ($(TESTS), 1)
	CARGO_BUILD_MODE += -F integration
	DEBUG = 1
endif

ifeq ($(DEBUG), 1)
	ASM_FLAGS += -g
	LINKER_FLAGS += -g
	TARGET_DIR := target/x86_64-unknown-none/debug
else
	CARGO_BUILD_MODE += --release
	TARGET_DIR := target/x86_64-unknown-none/release
endif

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
	cargo $(RUST_FLAGS) build $(CARGO_BUILD_MODE) --target x86_64-unknown-none
	cp $(TARGET_DIR)/libtao_os.a $(BUILDDIR)/kernel.o

clean:
	rm -rf build
	cargo clean
