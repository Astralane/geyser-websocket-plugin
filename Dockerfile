#build from latst rust version
FROM  --platform=linux/amd64 rust:1.79.0-slim-bullseye as build

# install libpq for diesel
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates libpq5 curl libpq-dev && \
    rm -rf /var/lib/apt/lists/* && \
    USER=root cargo new --bin app

WORKDIR /app

# copy files to build

COPY . .

# rebuild app with project source
RUN cargo build

RUN chmod +x install.sh && \
    ./install.sh

RUN PATH="/root/.local/share/solana/install/active_release/bin:$PATH"

#run
CMD ["bash"]
