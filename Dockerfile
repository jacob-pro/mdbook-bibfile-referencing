FROM ubuntu:22.04
WORKDIR /workdir

# Install useful tools for CI
ENV DEBIAN_FRONTEND=noninteractive
RUN apt update && apt install curl wget git build-essential nodejs npm -y
RUN npm install --save-dev --save-exact prettier

# Install Pandoc
RUN wget -O pandoc.deb https://github.com/jgm/pandoc/releases/download/2.19.2/pandoc-2.19.2-1-amd64.deb
RUN dpkg -i pandoc.deb
RUN rm pandoc.deb
RUN pandoc --version

# Install Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install mdbook and utilities
RUN cargo install mdbook
RUN cargo install mdbook-linkcheck

# Install mdbook-bibfile-referencing
RUN mkdir /build
COPY ./ /build
RUN cargo install --locked --path /build
RUN rm -rf /build

ENTRYPOINT mdbook
