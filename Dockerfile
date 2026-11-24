# Compilación
FROM rust:1-bookworm AS compilar
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release --locked -p faro-servidor -p faro-cli

# Ejecución
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --uid 10001 faro
WORKDIR /app
COPY --from=compilar /src/target/release/faro-servidor /src/target/release/faro /usr/local/bin/
COPY datos/catalogo ./datos/catalogo
COPY datos/solicitudes ./datos/solicitudes
COPY datos/estandar ./datos/estandar
COPY datos/referencia ./datos/referencia
COPY datos/ejemplos ./datos/ejemplos
RUN mkdir -p /app/datos/privado && chown -R faro /app/datos
USER faro
ENV FARO_PUERTO=8080
EXPOSE 8080
CMD ["faro-servidor"]
