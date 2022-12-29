FROM rust:1.61.0

WORKDIR /usr/src/social-music
COPY . .

RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.3/install.sh | bash
RUN nvm install 16

RUN cargo install --path .

CMD ["social-music"]