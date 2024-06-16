# Use latest stable channel SDK.
FROM dart:3.4 AS build

# Work Directory
WORKDIR /app

COPY . .

# Install the Vania cli from pub.dev
RUN dart pub global activate vania_cli

# Get dependencies
RUN dart pub get

# Copy app source code (except anything in .dockerignore) and AOT compile app.


# 📦 Create a production build
RUN dart pub get --offline


RUN vania build

# Build minimal serving image from AOT-compiled `/server`
# and the pre-built AOT-runtime in the `/runtime/` directory of the base image.
FROM scratch

COPY --from=build /runtime/ /
COPY --from=build /app/bin/server /bin/server
COPY --from=build /app/.env /

# Expose the server port (useful for binding)
EXPOSE 3000

# Start server.
CMD ["/bin/server"]