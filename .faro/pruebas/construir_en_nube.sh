#!/bin/bash
# Construye la imagen de FARO en esta nube: copia del Dockerfile del repositorio + certificado del proxy y espejo de imágenes.
S=${S:?defina S con una carpeta temporal}
cd /home/user/faro-ine-2026 || exit 1
sed -e 's#^FROM rust:1-bookworm AS compilar#FROM mirror.gcr.io/library/rust:1-bookworm AS compilar\nCOPY --from=ca ca-bundle.crt /ca.crt\nENV CARGO_HTTP_CAINFO=/ca.crt SSL_CERT_FILE=/ca.crt#' \
    -e 's#^FROM debian:bookworm-slim#FROM mirror.gcr.io/library/debian:bookworm-slim\nCOPY --from=ca ca-bundle.crt /ca.crt\nRUN sed -i "s,http://,https://," /etc/apt/sources.list.d/debian.sources \&\& echo "Acquire::https::CAInfo \\"/ca.crt\\";" > /etc/apt/apt.conf.d/99ca#' Dockerfile > $S/Dockerfile.nube
docker build --network host --build-context ca=$S/ca --build-arg HTTPS_PROXY=$HTTPS_PROXY --build-arg https_proxy=$HTTPS_PROXY --build-arg NO_PROXY="$NO_PROXY" --build-arg no_proxy="$NO_PROXY" -f $S/Dockerfile.nube -t faro-ine-2026-faro . > $S/build.log 2>&1 || { tail -20 $S/build.log; exit 1; }
[ "$1" = limpio ] && docker compose down -v >/dev/null 2>&1
docker compose up -d --no-build >/dev/null 2>&1
for i in $(seq 1 30); do curl -sf --noproxy '*' http://localhost:8082/api/estado >/dev/null && { echo listo; exit 0; }; perl -e 'sleep 2'; done; echo "no respondió"; exit 1
