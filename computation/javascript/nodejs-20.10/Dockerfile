FROM node:20.10-alpine

WORKDIR /app

ADD package.json /app/
RUN npm i
ADD . /app/

EXPOSE 3000
CMD ["npm", "start"]
