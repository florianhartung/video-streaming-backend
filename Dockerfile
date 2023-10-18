# Rust as the base image
FROM rust:1.73

# 1. Create a new empty shell project
RUN USER=root cargo new --bin video_streaming
WORKDIR /video_streaming

# 2. Copy our manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# 3. Build only the dependencies to cache them
RUN cargo build --release
RUN rm src/*.rs

# 4. Now that the dependency is built, copy your source code
COPY ./src ./src
COPY ./static ./static

# 5. Build for release.
RUN rm ./target/release/deps/video_streaming*
RUN cargo install --path .

EXPOSE 8080/TCP

CMD ["video-streaming"]