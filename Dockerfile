FROM ubuntu:20.04

ENV DEBIAN_FRONTEND=noninteractive
RUN apt update && apt install curl pandoc pandoc-citeproc git build-essential -y
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN cargo install mdbook
RUN mkdir /build
COPY ./ /build
RUN cargo install --locked --path /build
RUN rm -rf /build
WORKDIR /workdir
ENTRYPOINT mdbook
