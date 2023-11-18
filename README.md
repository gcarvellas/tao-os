# Tao OS

A hobbyist 64-bit operating system written in Rust. The goal of this project is to learn about Operating Systems development. 

## Building Tao OS ISO file

### Requirements:
- Rust
- nasm
- grub
- Docker

### Steps

1. Run `docker build -t tao-os-builder`. You only need to run this once.
2. Run `make dclean`
3. Run `make docker` 

## Running with QEMU

After building the ISO file, run `qemu-system-x86_64 -cdrom ./build/tao-os.iso`

## Features

- [x] Printing with VGA Text Mode

## TODO:

- Heap
- GDT/IDT
- TSS
- Paging
- FAT16
- Keyboard driver
- Processes/Tasks (User Programs)

- The docker builder makes the build files as root. Make it the current user
- Replace makefile with cargo.toml
- Have production build steps as well as debug
- Update the volatile crate (Replaces Volatile with VolatilePtr)
- Support colored printing
- Replace lazy_static with once_cell
