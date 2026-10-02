FROM oven/bun:1-alpine

WORKDIR /app
COPY . .

RUN bun install

EXPOSE 5001
CMD ["bun", "run", "main.js"]
