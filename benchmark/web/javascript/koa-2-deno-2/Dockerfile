FROM denoland/deno:2.1.12 AS builder
WORKDIR /app
COPY . .
RUN deno install && deno cache main.js

FROM denoland/deno:2.1.12 AS runtime
WORKDIR /app
COPY --from=builder /app .

EXPOSE 3000
CMD ["deno", "run", "--allow-net", "--allow-env", "--allow-read", "main.js"]
