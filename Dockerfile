FROM rust:latest as builder


RUN set -eux; \
  apt-get update; \
  apt-get install -y --no-install-recommends \
  clang

WORKDIR /app
COPY . /app
COPY etc/cargo/config.toml /root/.cargo/config.toml

RUN --mount=type=cache,target=/root/.cargo/registry --mount=type=cache,target=/app/target cargo build --release --bin a2a && cp /app/target/release/a2a /app/a2a


FROM registry.cn-beijing.aliyuncs.com/kanhq-dev/a2a:base


COPY --from=builder /app/a2a/a2a /usr/bin/a2a
# COPY --from=builder /lib/x86_64-linux-gnu/libssl* /lib/x86_64-linux-gnu/
# COPY --from=builder /lib/x86_64-linux-gnu/libcrypto* /lib/x86_64-linux-gnu/
# COPY --from=builder /etc/ssl/certs /etc/ssl/certs


WORKDIR /a2a
CMD [ "a2a", "serve", "-l", "0.0.0.0:30030", "--no-ui"]