FROM alpine
WORKDIR /app

# Install cURL
RUN apk update
RUN apk add curl

# Add wait script & make it executable
COPY wait.sh /app/wait.sh
RUN chmod +x wait.sh

# Wait
CMD ./wait.sh
