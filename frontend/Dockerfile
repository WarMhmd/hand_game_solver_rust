# Build stage
FROM node:20-alpine as builder

WORKDIR /app

# Copy package files
COPY package*.json ./

# Install dependencies
RUN npm ci

# Copy source code
COPY . .

# Build the application
RUN npm run build

# Production stage
FROM node:20-alpine

WORKDIR /app

# Install serve to host the static files and wget for health checks
RUN npm install -g serve && \
    apk add --no-cache wget

# Copy built assets from builder
COPY --from=builder /app/dist ./dist

# Expose port 3001
EXPOSE 3001

# Start serve
CMD ["serve", "-s", "dist", "-l", "3001"]
