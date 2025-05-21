FROM rust:latest AS build-phase-1

RUN mkdir /dodo

WORKDIR /dodo

RUN rustup target add x86_64-unknown-linux-musl

COPY . .

RUN cargo build --release --target x86_64-unknown-linux-musl

RUN mv /dodo/target/x86_64-unknown-linux-musl/release/dodo-payment / && strip /dodo-payment

FROM ubuntu:latest AS build-phase-2

COPY --from=build-phase-1 /dodo-payment /

RUN apt-get update -y && apt-get install wget -y

RUN wget https://go.dev/dl/go1.24.3.linux-amd64.tar.gz

RUN tar -C /usr/local -xzf go1.24.3.linux-amd64.tar.gz && rm -rf go1.24.3.linux-amd64.tar.gz

RUN /usr/local/go/bin/go install github.com/pressly/goose/v3/cmd/goose@latest

FROM alpine:latest AS main-phase

WORKDIR /

COPY --from=build-phase-1 /dodo/migrations /

COPY --from=build-phase-2 /dodo-payment .

COPY --from=build-phase-2 /root/go/bin/goose .

EXPOSE 11000

CMD [ "/dodo-payment" ]