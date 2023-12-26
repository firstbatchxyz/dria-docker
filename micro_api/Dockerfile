# Using Node 20
FROM node:20
WORKDIR /app

# Install dependencies
COPY package.json /app 
COPY yarn.lock /app
COPY tsconfig.json /app 
COPY src /app/src
RUN yarn install 

# Compile & build
RUN yarn compile

# Run Micro
CMD ["npx", "micro", "./build/index.js"]
EXPOSE 3000
