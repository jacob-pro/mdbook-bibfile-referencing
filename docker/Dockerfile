FROM ubuntu:20.04

ENV DEBIAN_FRONTEND=noninteractive
RUN apt update && apt install curl pandoc pandoc-citeproc build-essential -y
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN cargo install mdbook
RUN cargo install --git https://github.com/jacob-pro/mdbook-bibfile-referencing
WORKDIR /workdir
ENTRYPOINT mdbook
