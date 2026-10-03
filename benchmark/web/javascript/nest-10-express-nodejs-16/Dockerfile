FROM node:16-alpine

WORKDIR /app
COPY . .

RUN npm install -g @nestjs/cli
RUN npm install

RUN nest build

CMD ["node", "dist/main.js"]
