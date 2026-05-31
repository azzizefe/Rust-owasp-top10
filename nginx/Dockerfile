FROM nginx:1.25-alpine

# Install OpenSSL to generate development certificates automatically
RUN apk add --no-cache openssl

# Prepare SSL directory
RUN mkdir -p /etc/nginx/ssl

# Generate self-signed fallback certificate for local development
RUN openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
    -keyout /etc/nginx/ssl/owasp_lab.key \
    -out /etc/nginx/ssl/owasp_lab.crt \
    -subj "/C=TR/ST=Istanbul/L=Center/O=OWASP Lab/CN=localhost"

# Copy Nginx config
COPY nginx.conf /etc/nginx/nginx.conf
COPY cloudflare-shield.conf /etc/nginx/cloudflare-shield.conf

EXPOSE 80 443
