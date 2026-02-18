FROM alpine:3.18

# Simulate build environment
RUN apk add --no-cache nodejs npm

WORKDIR /app

# Step 1: Copy package.json
COPY package.json .

# Step 2: Install dependencies
RUN npm install

# Step 3: Copy source
COPY src ./src

# Step 4: Build
RUN npm run build

# Final config
ENV NODE_ENV=production
CMD ["npm", "start"]
