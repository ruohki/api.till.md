# 1: Build the exe
FROM rustlang/rust:nightly-alpine as builder
WORKDIR /usr/src

# 1a: Prepare for static linking
#RUN apt-get update
#RUN apt-get dist-upgrade -y
#RUN apt-get install -y musl-tools
RUN apk add --no-cache musl musl-dev openssl openssl-dev
RUN rustup target add x86_64-unknown-linux-musl

# 1b: Download and compile Rust dependencies (and store as a separate Docker layer)
RUN USER=root cargo new api
WORKDIR /usr/src/api
COPY . .
RUN cargo build --target x86_64-unknown-linux-musl --release

# 2: Copy the exe and extra files ("static") to an empty Docker image
FROM scratch
COPY --from=builder /usr/src/api/target/x86_64-unknown-linux-musl/release/api .
USER 1000
CMD ["./api"]