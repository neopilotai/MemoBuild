FROM golang:1.24-alpine AS builder
ENV CGO_ENABLED=0
WORKDIR /backend
COPY extension/backend/go.* .
RUN go mod download
COPY extension/backend/. .
RUN go build -trimpath -ldflags="-s -w" -o bin/service

FROM --platform=$BUILDPLATFORM node:24-alpine AS client-builder
WORKDIR /ui
COPY extension/ui/package.json extension/ui/package-lock.json ./
RUN npm ci --legacy-peer-deps
COPY extension/ui/. .
RUN npm run build

FROM alpine
LABEL org.opencontainers.image.title="MemoBuild - Incremental Build Farm" \
    com.docker.desktop.extension.api.version="0.4.2"

# Stub for memobuild
RUN echo '#!/bin/sh' > /usr/local/bin/memobuild && \
    echo 'echo "MemoBuild CLI (Stub)"' >> /usr/local/bin/memobuild && \
    chmod +x /usr/local/bin/memobuild

COPY --from=builder /backend/bin/service /
COPY extension/docker-compose.yaml .
COPY extension/metadata.json .
COPY extension/docker.svg .
COPY --from=client-builder /ui/build ui
CMD /service -socket /run/guest-services/backend.sock
