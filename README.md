# Tao OS

A hobbyist 64-bit operating system written in Rust. The goal of this project is to learn about Operating Systems development. 

## Building Tao OS ISO file

### Requirements:
- Rust
- nasm
- grub
- Docker

### Steps

1. Run `docker build -t tao-os-builder .`
2. Run `make docker` 

## Running with QEMU

After building the ISO file, run `qemu-system-x86_64 -cdrom ./build/tao-os.iso`

## TODO:

- [ ] Printing
- [ ] Heap
- [ ] The docker builder makes the build files as root. Make it the current user
- [ ] Replace makefile with cargo.toml
- [ ] Have production build steps as well as debug
