FROM rust:1.61.0

WORKDIR /usr/src/social-music
COPY . .

RUN cargo install --path .

CMD ["social-music"]