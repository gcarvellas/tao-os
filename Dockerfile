FROM ubuntu:22.04

ARG USER=docker
ARG UID=1000
ARG GID=1000
# default password for user
ARG PW=docker

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

RUN useradd -m ${USER} --uid=${UID} && echo "${USER}:${PW}" | \
      chpasswd

USER ${UID}:${GID}
WORKDIR /home/${USER}

ENV RUSTUP_HOME=/home/${USER}/rustup
ENV CARGO_HOME=/home/${USER}/cargo
RUN /bin/bash -c 'curl --proto '=https' --tlsv1.3 https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain=nightly'

ENV PATH=$PATH:/home/${USER}/cargo/bin
RUN rustup target add x86_64-unknown-none
