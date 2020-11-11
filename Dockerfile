FROM rust:1.47.0 as build-env
WORKDIR /app
ENV USER=root

# Create a dummy project
RUN cargo init --bin

# Copy in dependencies from the actual project
COPY Cargo.toml Cargo.lock ./

# Download and compile dependencies
RUN cargo build --release

# Remove the unnecessary files from the dummy project
RUN rm -f ./target/release/discord-server-manager*
RUN rm -rf ./src

# Copy the source from the actual project
COPY src ./src

# Compile the actual project
RUN cargo build --release

FROM gcr.io/distroless/cc 
COPY --from=build-env /app/target/release/discord-server-manager /
ENTRYPOINT ["/discord-server-manager"]
