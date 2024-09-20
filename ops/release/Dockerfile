FROM alpine:latest

ARG TZ_LOCALE="Europe/Brussels"

# DEPENDENCIES
RUN apk add --update --no-cache \
    dumb-init \
    ca-certificates

# TZ
RUN set -eux && \
    apk add --update --no-cache tzdata && \
    cp "/usr/share/zoneinfo/$TZ_LOCALE" /etc/localtime && \
    echo "$TZ_LOCALE" > /etc/timezone && \
    apk del tzdata

# User & Group
RUN addgroup -S polar && adduser -S -G polar -s /sbin/nologin polar

# Binaries
ADD --chown=polar:polar --chmod=770 ../../target/release/polar /usr/local/bin
ADD --chown=polar:polar --chmod=660 ../../resources/polar.toml /etc/polar

EXPOSE 8000
VOLUME /etc/polar
USER polar

ENTRYPOINT ["dumb-init", "--"]
CMD ["polar", "serve"]
