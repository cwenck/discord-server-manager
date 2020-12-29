FROM gcr.io/distroless/cc
COPY ./target/release/discord-server-manager /
ENTRYPOINT ["/discord-server-manager"]
