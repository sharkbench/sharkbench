# trixie: µWebSockets.js ships glibc binaries needing GLIBC >= 2.38 (no Alpine, no bookworm),
# and it is installed from GitHub, so git is needed
FROM node:26-trixie-slim

RUN apt-get update && apt-get install -y --no-install-recommends git ca-certificates && rm -rf /var/lib/apt/lists/*

# npm 12 refuses git dependencies unless told otherwise
ENV NPM_CONFIG_ALLOW_GIT=all

WORKDIR /app
COPY . .

RUN npm install

EXPOSE 3000
CMD ["node", "main.js"]
