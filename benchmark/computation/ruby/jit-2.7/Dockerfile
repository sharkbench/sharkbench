FROM ruby:2.7-slim

WORKDIR /app
COPY . .

EXPOSE 5001
CMD ["ruby", "--jit", "main.rb"]
