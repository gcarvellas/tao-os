FROM ubuntu:22.04

RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y \
    nasm \
    xorriso \
    grub-pc-bin \
    grub-common \
    gcc \
    make \
    curl

ENV RUSTUP_HOME=/rustup
ENV CARGO_HOME=/cargo
RUN /bin/bash -c 'curl --proto '=https' --tlsv1.3 https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain=nightly'

ENV PATH=$PATH:/cargo/bin
RUN rustup target add x86_64-unknown-none
