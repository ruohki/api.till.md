# 1: Build the exe
FROM rustlang/rust:nightly-alpine as builder
WORKDIR /usr/src

# 1a: Prepare for static linking
RUN apt-get update && \
    apt-get dist-upgrade -y && \
    apt-get install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl

# 1b: Download and compile Rust dependencies (and store as a separate Docker layer)
RUN USER=root cargo new api
WORKDIR /usr/src/api
COPY Cargo.toml Cargo.lock ./
RUN cargo install --target x86_64-unknown-linux-musl --path .

# 1c: Build the exe using the actual source code
COPY src ./src
RUN cargo install --target x86_64-unknown-linux-musl --path .

# 2: Copy the exe and extra files ("static") to an empty Docker image
FROM scratch
COPY --from=builder /usr/local/cargo/bin/api .
#COPY static .
USER 1000
CMD ["./api"]