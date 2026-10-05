FROM node:22-alpine

WORKDIR /app
COPY . .

RUN npm install -g @nestjs/cli@11
RUN npm install

RUN nest build

CMD ["node", "dist/main.js"]
