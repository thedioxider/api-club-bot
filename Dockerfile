FROM rust:1.86

# set a directory for the app
WORKDIR /usr/src/bot
# copy all the files to the container
COPY . .

# install app-specific dependencies
RUN cargo install --path .

# app command
CMD ["api-club-bot"]
