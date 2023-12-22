# Using Node 20
FROM node:20
WORKDIR /app

# Install dependencies
COPY package.json /app 
COPY yarn.lock /app
RUN yarn install 

# Run Micro
COPY . /app 
CMD ["npx", "micro", "./build/src/index.js"]
EXPOSE 3000
