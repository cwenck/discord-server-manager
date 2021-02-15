FROM gcr.io/distroless/cc
COPY ./target/release/discord-server-manager /
RUN chmod +x /discord-server-manager
ENTRYPOINT ["/discord-server-manager"]
