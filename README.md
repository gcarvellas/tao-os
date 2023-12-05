# Tao OS

A hobbyist 64-bit operating system written in Rust. The goal of this project is to learn about Operating Systems development and make a usable operating system. 

## Building Tao OS ISO file

### Requirements:
- Rust
- nasm
- grub

Or alternatively, you can use Docker

### Steps

1. Run `docker build --build-arg USER=$USER --build-arg UID=$UID --build-arg GID=$GID --build-arg PW=docker -t tao-os-builder .`. You only need to run this once.
2. Run `make clean`
3. Run `make docker` 

## Running with QEMU

After building the ISO file, run `qemu-system-x86_64 -cdrom ./build/tao-os.iso`

## Features

- [x] Printing with VGA Text Mode
- [x] Memory Allocation with First Fit Algorithm
- [x] Interrupts

## TODO:

### Core Features
- Paging
- FAT16
- Keyboard driver
- Mouse driver
- network driver
- sound driver
- graphics
- Processes/Tasks (User Programs)

### Cleanup/Improvements
- Replace First Fit with Slab Allocation algorithm
- Clear the neverending backlog of TODO comments

### Minor Cleanup/Improvements
- Add more interrupts and improve the interrupt abstractions
- Replace makefile with cargo.toml
- Have production build steps as well as debug
- Update the volatile crate (Replaces Volatile with VolatilePtr)
- Support colored printing
- Replace lazy_static with once_cell
- Error checking with cpuid
- Make doesn't work natively on gentoo
