FROM rust:latest

RUN mkdir /dodo

WORKDIR /dodo

RUN rustup target add x86_64-unknown-linux-musl

COPY . .

RUN cargo build --release --target x86_64-unknown-linux-musl

RUN mv /dodo/target/x86_64-unknown-linux-musl/release/dodo-payment / && strip /dodo-payment

RUN rm -rf /dodo

EXPOSE 11000

CMD [ "/dodo-payment" ]