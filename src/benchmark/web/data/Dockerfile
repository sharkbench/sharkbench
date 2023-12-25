FROM nginx:1.25.3-alpine
COPY config.conf /etc/nginx/conf.d/default.conf
COPY data.json /usr/share/nginx/html

# Start server
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
