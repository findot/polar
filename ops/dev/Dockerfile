FROM alpine:latest

ARG TZ_LOCALE="Europe/Brussels"
ARG USER_ID
ARG GROUP_ID

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUSTUP_URL="https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-musl/rustup-init" \
    RUSTFLAGS="-C target-feature=-crt-static" \
    POLAR_PROFILE="dev"

# DEPENDENCIES
RUN apk add --update --no-cache \
    util-linux \
    dumb-init \
    ca-certificates \
    gcc \
    musl-dev \
    libpq-dev \
    inotify-tools

RUN addgroup -S -g $GROUP_ID polar \
    && adduser -S -u $USER_ID -G polar polar

# CARGO & RUST
RUN set -eux && \
    wget "$RUSTUP_URL" && \
    chmod +x rustup-init && \
    ./rustup-init -y --no-modify-path --default-toolchain stable && \
    rm rustup-init && \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME

# TZ
RUN set -eux && \
    apk add --update --no-cache tzdata && \
    cp "/usr/share/zoneinfo/$TZ_LOCALE" /etc/localtime && \
    echo "$TZ_LOCALE" > /etc/timezone && \
    apk del tzdata

VOLUME /usr/src/app
VOLUME /usr/local/cargo/registry

ADD --chmod=770 --chown=polar entrypoint.sh /

EXPOSE 8000

ENTRYPOINT ["dumb-init", "--"]
CMD ["/entrypoint.sh"]
