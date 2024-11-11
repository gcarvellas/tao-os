# Tao OS

A hobbyist x86_64 operating system written in Rust. This OS is a personal project to learn about OS development. 
This OS is WIP, but the end goal is to have a working wm, with some basic user programs and kernel drivers. From this point, this OS can be used to experiment with unique OS implementation ideas.

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
- [x] Basic Paging
- [x] ATA PIO Hard Disk Reading
- [x] FAT16 Reading

## TODO:

### Core Features
- Keyboard driver
- Interrupt-Driven Async
- Processes/Multitasking
- Elf loader/User Programs
- Testing
- Mouse driver
- network driver
- sound driver
- graphics
- Processes/Tasks (User Programs)
- DMA driver
- RamFS
- PCI Scan

### Cleanup/Improvements
- Replace First Fit with Slab Allocation algorithm
- Clear the neverending backlog of TODO comments and unimplemented!() macros
- Paging improvements
- Swap implementation
- Fat16 writing

### Minor Cleanup/Improvements
- Pay attention to which orderings I'm using for address loadings
- Add more interrupts and improve the interrupt abstractions
- Update the volatile crate (Replaces Volatile with VolatilePtr)
- Support colored printing
- Error checking with cpuid in the bootloader
- Use proper locking instead of spin locks (lock api)
- Stack trace
- properly use rust test crate
