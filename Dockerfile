
# ---------- build ----------
ARG KOVI_CONF_TEMPLATE_NAME='template.kovi.conf.toml'
ARG KOVI_CONF_USE_NAME='kovi.conf.toml'
# bun init
FROM oven/bun:1.2.22-alpine AS bun-env
WORKDIR /app
COPY .scripts ./.scripts
WORKDIR /app/.scripts
RUN bun install

# prebuild scripts
ENV FOR_CMD="1"
ENV SCRIPT_WORK_DIR="/app"
# pass

# rust bin build
FROM rust:1.90 AS rust-env
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY plugins ./plugins
RUN cargo build --release

# post build scripts
FROM bun-env AS bun-final
WORKDIR /app
ARG KOVI_CONF_TEMPLATE_NAME
ARG KOVI_CONF_USE_NAME
# post build可以传入env来控制kovi生成的配置文件
# .eg MAIN_ADMIN=10001
ARG MAIN_ADMIN
# .eg ADMINS=10002,10003,10004
ARG ADMINS
# .eg HOST=0.0.0.0
ARG HOST='host.docker.internal'
# .eg PORT=3303
ARG PORT
# .eg ACCESS_TOKEN=hahxixixi
ARG ACCESS_TOKEN
# .eg SECURE=1
ARG SECURE

ENV MAIN_ADMIN=${MAIN_ADMIN} \
    ADMINS=${ADMINS} \
    HOST=${HOST} \
    PORT=${PORT} \
    ACCESS_TOKEN=${ACCESS_TOKEN} \
    SECURE=${SECURE}
# 拷贝构建产物
COPY --from=rust-env /app/target/release/nya-bot /app/
ENV FOR_CMD="1"
ENV SCRIPT_WORK_DIR="/app"
# 拷贝postbuild需要的
ENV KOVI_CONF_TEMPLATE_NAME=$KOVI_CONF_TEMPLATE_NAME
ENV KOVI_CONF_USE_NAME=$KOVI_CONF_USE_NAME
COPY $KOVI_CONF_TEMPLATE_NAME ./$KOVI_CONF_TEMPLATE_NAME
# 跑脚本
RUN bun .scripts/postbuild.ts

# ---------- runtime ----------
FROM debian:bookworm-slim
LABEL authors="rdjksp"
WORKDIR /app
# 最终输出的文件 可执行文件和预先配置的config
ARG KOVI_CONF_USE_NAME
COPY --from=bun-final /app/nya-bot /app/$KOVI_CONF_USE_NAME /app/
# 运行时依赖
RUN apt-get update && apt-get install -y libssl3 ca-certificates

CMD ["/app/nya-bot"]
