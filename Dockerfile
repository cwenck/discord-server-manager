FROM ubuntu:latest as BUILDER
COPY ./target/release/discord-server-manager /
RUN chmod +x /discord-server-manager

FROM gcr.io/distroless/cc
COPY --from=BUILDER /discord-server-manager /
ENTRYPOINT ["/discord-server-manager"]
