FROM rust:1.87

WORKDIR /usr/src/myapp

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./

# install toolchain
RUN rustup default stable

COPY . .

# compile the application in debug mode
RUN cargo build

# spawn a shell in the container
CMD ["bash"]
# CMD ["cargo", "test", "--all", "--", "--nocapture"]
