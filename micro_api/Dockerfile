# Using Node 20
FROM node:20 as base
WORKDIR /app

COPY . .

RUN yarn set version berry

# yarn install might take time when a non-readonly volume is attached (see https://github.com/yarnpkg/yarn/issues/7747)
# ~700 seconds on an M2 Macbook Air for example
RUN yarn install

# Build code
FROM base as builder
WORKDIR /app
RUN yarn build

# Install prod dependencies
FROM base as dependencies
WORKDIR /app
RUN yarn workspaces focus --production

FROM node:20
WORKDIR /app
COPY --from=builder /app/build ./build
COPY --from=dependencies /app/node_modules ./node_modules

EXPOSE 3000

CMD ["node", "./build/index.js"]


