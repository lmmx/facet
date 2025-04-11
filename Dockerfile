FROM rust:1.86-slim-bullseye as builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    git \
    pkg-config \
    libssl-dev \
    build-essential \
    clang \
    llvm \
    lld \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# Install Rust nightly
RUN rustup toolchain install nightly

# Install the specific nightly version needed for Miri (matches Justfile)
RUN rustup toolchain install nightly-2025-04-05

# Install Miri on the specific nightly version
RUN rustup component add miri rust-src --toolchain nightly-2025-04-05

# Add rust-src, clippy, and rustfmt to stable toolchain
RUN rustup component add rust-src clippy rustfmt

# Install just and cargo-nextest
RUN cargo install --locked just cargo-nextest

# Set environment variables
ENV CARGO_INCREMENTAL=0

# Add the cargo bin directory to PATH
ENV PATH="/root/.cargo/bin:${PATH}"

# Create a work directory
WORKDIR /app

CMD ["bash"]