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

EXPOSE 8000
VOLUME /etc/polar

ADD entrypoint.sh /
ADD ../../target/release/polar /usr/local/bin
ADD ../../resources/polar.toml /etc/polar
RUN chmod a+x /entrypoint.sh

ENTRYPOINT ["dumb-init", "--"]
CMD ["polar", "serve"]
