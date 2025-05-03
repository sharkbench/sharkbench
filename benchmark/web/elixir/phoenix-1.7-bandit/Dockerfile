FROM elixir:1.16-slim AS builder

WORKDIR /app

# Required by Elixir 1.18+ (OTP 26+)
RUN apt update && apt install -y ca-certificates

# Install hex package manager and Phoenix framework
RUN mix local.hex --force && \
    mix local.rebar --force && \
    mix archive.install hex phx_new --force

COPY mix.exs ./
COPY config config

ENV MIX_ENV=prod

RUN mix deps.get && mix deps.compile

COPY lib lib

RUN mix do compile, release

FROM debian:bookworm-slim AS runtime
RUN apt update && apt install -y libssl-dev

WORKDIR /app

COPY --from=builder /app/_build/prod/rel/benchmark ./

# dummy key
ENV SECRET_KEY_BASE="Cm+q1aBiaPxplQH8i4W/dIJdU3ldL4uysgnseOMnA7G5McpRgbommCoOF/N/rMnj"

ENV PHX_SERVER=true
ENV PORT=3000

EXPOSE 3000
ENTRYPOINT ["/app/bin/benchmark"]
CMD ["start"]
