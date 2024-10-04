# Tao OS

A hobbyist 64-bit operating system written in Rust. The goal of this project is to learn about Operating Systems development and make a usable operating system. 

## Building Tao OS ISO file

### Requirements:
- Rust
- nasm
- grub

## Running with QEMU

After building the ISO file, run `qemu-system-x86_64 -cdrom ./build/tao-os.iso`

## Features

- [x] Printing with VGA Text Mode
- [x] Memory Allocation with First Fit Algorithm
- [x] Interrupts
- [x] Paging

## TODO:

### Core Features
- Paging improvements
- FAT16
- Keyboard driver
- Mouse driver
- network driver
- sound driver
- graphics
- Processes/Tasks (User Programs)
- Async
- Testing

### Cleanup/Improvements
- Replace First Fit with Slab Allocation algorithm
- Clear the neverending backlog of TODO comments

### Minor Cleanup/Improvements
- Pay attention to which orderings I'm using for address loadings
- use proper errors instead of just ErrorCode. 
- Add more interrupts and improve the interrupt abstractions
- Replace makefile with cargo.toml
- Have production build steps as well as debug
- Update the volatile crate (Replaces Volatile with VolatilePtr)
- Support colored printing
- Error checking with cpuid in the bootloader
