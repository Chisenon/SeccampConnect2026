FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

# ── Create non-root user ─────────────────────────────────────────────────────
RUN useradd -m -s /bin/bash builder

# ── System dependencies ──────────────────────────────────────────────────────
RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    git \
    xz-utils \
    libssl-dev \
    pkg-config \
    libusb-1.0-0 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# ── Rustup with stable (needed to build espup itself) ───────────────────────
#   stable is used only to compile the espup binary.
#   The actual ESP32S3 build uses the Xtensa toolchain installed by espup,
#   selected via rust-toolchain.toml (channel = "esp").
USER builder
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain stable

ENV PATH="/home/builder/.cargo/bin:${PATH}"

# ── espup: install Xtensa Rust toolchain (version matches nix/rust-esp.nix) ──
#   This installs:
#     - Xtensa-enabled rustc/cargo from esp-rs/rust-build v1.90.0.0
#     - xtensa-esp-elf GCC cross-compiler
#     - LLVM/Clang for Xtensa
#   and generates ~/export-esp.sh with required env vars.
RUN cargo install espup
RUN espup install --toolchain-version 1.90.0.0

# ── esp32ulp-elf & xtensa-esp-elf-gdb ────────────────────────────────────────
#   These are not included in espup; downloaded directly from Espressif releases.
#   Versions match those shipped with ESP-IDF v5.3/5.4.
USER root
RUN mkdir -p /opt/esp-tools && \
    curl -L "https://github.com/espressif/binutils-gdb/releases/download/esp-gdb-v14.2_20240403/xtensa-esp-elf-gdb-14.2_20240403-x86_64-linux-gnu.tar.gz" \
        | tar -xz -C /opt/esp-tools && \
    curl -L "https://github.com/espressif/binutils-gdb/releases/download/esp32ulp-elf-2.38_20240113/esp32ulp-elf-2.38_20240113-linux-amd64.tar.gz" \
        | tar -xz -C /opt/esp-tools && \
    chmod -R a+rx /opt/esp-tools

ENV PATH="/opt/esp-tools/xtensa-esp-elf-gdb/bin:/opt/esp-tools/esp32ulp-elf/bin:${PATH}"

# ── Bake the exported env vars into the image ─────────────────────────────────
#   export-esp.sh sets PATH, LIBCLANG_PATH, CLANG_PATH, etc.
#   We re-prepend /opt/esp-tools paths afterwards because export-esp.sh
#   may reset PATH and drop our custom entries.
RUN printf '#!/bin/bash\n. /home/builder/export-esp.sh\nexport PATH="/opt/esp-tools/xtensa-esp-elf-gdb/bin:/opt/esp-tools/esp32ulp-elf/bin:$PATH"\nexec "$@"\n' \
    > /usr/local/bin/esp-env && \
    chmod +x /usr/local/bin/esp-env

# Auto-load environment on shell startup
RUN printf 'if [ -f /home/builder/export-esp.sh ]; then\n  . /home/builder/export-esp.sh\n  export PATH="/opt/esp-tools/xtensa-esp-elf-gdb/bin:/opt/esp-tools/esp32ulp-elf/bin:$PATH"\nfi\n' \
    >> /home/builder/.bashrc

# ── Pre-fetch cargo registry dependencies (cache layer) ──────────────────────
#   Copy only the manifest files so this layer is rebuilt only when deps change.
WORKDIR /work
RUN chown -R builder:builder /work
USER builder
COPY Cargo.toml build.rs rust-toolchain.toml ./
COPY rmk ./rmk
# Create a stub main so `cargo fetch` succeeds without the real source.
RUN mkdir -p src && echo 'fn main(){}' > src/main.rs
RUN . /home/builder/export-esp.sh && cargo fetch
# Remove the stub; the real source will come from the mounted volume.
RUN rm -rf src

WORKDIR /work
ENTRYPOINT ["/usr/local/bin/esp-env"]
CMD ["cargo", "build", "--release"]
