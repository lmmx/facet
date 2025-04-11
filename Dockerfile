FROM rust:1.86-slim-bullseye as builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    git \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Install Rust nightly
RUN rustup toolchain install nightly

# Install Miri
RUN rustup component add miri --toolchain nightly

# Install just and cargo-nextest
RUN cargo install --locked just cargo-nextest

# Set environment variables
ENV CARGO_INCREMENTAL=0

# Add the cargo bin directory to PATH
ENV PATH="/root/.cargo/bin:${PATH}"

# Create a work directory
WORKDIR /app

CMD ["bash"]