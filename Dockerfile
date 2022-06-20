FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive
RUN apt update && apt install curl pandoc pandoc-citeproc git build-essential nodejs npm -y
RUN pandoc --version
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN cargo install mdbook
RUN cargo install mdbook-linkcheck
RUN mkdir /build
COPY ./ /build
RUN cargo install --locked --path /build
RUN rm -rf /build
WORKDIR /workdir
RUN npm install --save-dev --save-exact prettier
ENTRYPOINT mdbook
