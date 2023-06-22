FROM rust:1 as builder

ARG SCHLUESSEL_VERSION
ENV SCHLUESSEL_VERSION = $SCHLUESSEL_VERSION

WORKDIR /app
COPY . /app
RUN cargo build --release

FROM gcr.io/distroless/cc
COPY --from=builder /app/target/release/schluessel /
EXPOSE 8080
CMD ["./schluessel"]
