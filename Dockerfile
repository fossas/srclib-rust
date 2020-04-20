FROM rust as builder

WORKDIR /tc-build
COPY src/ src/
COPY Cargo.* ./

RUN cargo build --release --locked

FROM rust

COPY --from=builder /tc-build/target/release/srclib-rust /usr/local/bin/

RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y git

WORKDIR /tc-test

COPY test-me.sh ./

CMD [ "/tc-test/test-me.sh" ]