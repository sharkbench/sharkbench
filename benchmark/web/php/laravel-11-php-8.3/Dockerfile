FROM php:8.3-fpm

# https://serverfault.com/questions/955883/php-fpm-is-slow-with-nginx
RUN apt-get update \
  && apt-get -y install --no-install-recommends \
	nginx \
    unzip \
    libzip-dev \
    libicu-dev \
    libonig-dev \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/* \
  && docker-php-ext-install zip \
    intl \
    zip \
    bcmath \
    opcache

WORKDIR /app

# Install Composer
COPY --from=composer:latest /usr/bin/composer /usr/bin/composer

COPY . .

# Install PHP dependencies
RUN COMPOSER_ALLOW_SUPERUSER=1 composer update && composer install --no-dev --optimize-autoloader

RUN chmod -R 777 storage
  
RUN mkdir -p var
RUN chown -R www-data:www-data var

COPY nginx.conf /etc/nginx/conf.d/custom.conf
COPY www.conf /usr/local/etc/php-fpm.d/www.conf

# Start Nginx and PHP-FPM
EXPOSE 3000
CMD service nginx start && php-fpm
