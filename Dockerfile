FROM rust:1.64

WORKDIR /usr/src/giant-utils
COPY . .

RUN cargo install --path .

CMD ["giant-utils"]

