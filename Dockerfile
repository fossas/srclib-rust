FROM rust

RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y git

WORKDIR /tc-build

# Pre-build dependencies so that cached builds are faster
COPY Cargo.* ./
COPY src/dummy_main.rs src/main.rs
RUN cargo build

# Overwrites the dummy_main
COPY src/ src/

CMD cargo run --frozen -- scan --repo x --subdir .