FROM oven/bun:1-alpine

WORKDIR /app
COPY . .

RUN bun install

EXPOSE 3000
CMD ["bun", "run", "main.js"]
