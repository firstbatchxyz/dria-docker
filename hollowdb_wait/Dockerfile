FROM alpine
WORKDIR /app

RUN apk update
RUN apk add curl

COPY wait.sh /app/wait.sh
RUN chmod +x wait.sh

CMD ./wait.sh
