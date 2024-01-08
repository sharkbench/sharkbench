FROM php:8.3-fpm

RUN apt-get update && apt-get install -y \
    nginx \
    libzip-dev \
    zip \
    && docker-php-ext-install zip

WORKDIR /app

# Install Composer
COPY --from=composer:latest /usr/bin/composer /usr/bin/composer

COPY . .

# Install PHP dependencies
RUN COMPOSER_ALLOW_SUPERUSER=1 composer update && composer install --no-dev --optimize-autoloader

RUN chown -R www-data:www-data var

COPY nginx.conf /etc/nginx/conf.d/custom.conf

# Start Nginx and PHP-FPM
EXPOSE 3000
CMD service nginx start && php-fpm
