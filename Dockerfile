# Use the official Rust image as a base
FROM rust:1.70 as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libasound2-dev \
    libudev-dev \
    vulkan-tools \
    vulkan-validationlayers \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy the source code
COPY . .

# Build the application
RUN cargo build --release

# Use a smaller base image for the runtime
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libasound2 \
    libudev1 \
    vulkan-tools \
    vulkan-validationlayers \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/sandk-offroad /usr/local/bin/

# Set up the runtime environment
ENV RUST_BACKTRACE=1
ENV VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/intel_icd.x86_64.json:/usr/share/vulkan/icd.d/radeon_icd.x86_64.json:/usr/share/vulkan/icd.d/nvidia_icd.json

# Run the application
CMD ["sandk-offroad"] 