FROM rust

RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y git

WORKDIR /tc-build
COPY src/ src/
COPY Cargo.* ./

RUN cargo build

CMD cargo run -- scan --repo x --subdir .