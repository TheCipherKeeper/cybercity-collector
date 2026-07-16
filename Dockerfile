FROM rust:1.96-slim AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=build /src/target/release/cybercity-collector /cybercity-collector
COPY config/example.toml /config/example.toml
USER nonroot:nonroot
ENTRYPOINT ["/cybercity-collector", "/config/example.toml"]
