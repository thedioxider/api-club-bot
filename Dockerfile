FROM rust:1.86

# set a directory for the app
WORKDIR /usr/src/bot
# copy all the files to the container
COPY . .

# install app-specific dependencies
RUN cargo install --path .

# provide bot token to teloxide
ARG BOT_TOKEN
ENV TELOXIDE_TOKEN=$BOT_TOKEN
# app command
CMD ["api-club-bot"]
