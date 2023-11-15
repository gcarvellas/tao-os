# Tao OS

A hobbyist 64-bit operating system written in Rust. The goal of this project is to learn about Operating Systems development. 

## Building Tao OS ISO file

### Requirements:
- Rust
- nasm
- grub

### Steps

1. Run `make clean`
2. Run `make all`

## Running with QEMU

After building the ISO file, run `qemu-system-x86_64 -cdrom ./build/tao-os.iso`

## TODO:

- [ ] Get the kernel booting (multiboot header invalid)
- [ ] Replace makefile with cargo.toml
