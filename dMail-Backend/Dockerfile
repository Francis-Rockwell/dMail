FROM rust:latest
RUN rustup install nightly && rustup default nightly

WORKDIR /user/dMail

COPY .cargo ./cargo/
COPY src ./src/
COPY tests ./tests/
COPY Cargo.lock ./
COPY Cargo.toml ./
RUN mkdir config && cargo build --release

EXPOSE 8080

CMD ["./target/release/dMail"]