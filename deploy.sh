#!/bin/bash

# 🚀 Polkavex One-Click Deploy Script
# This script sets up the entire Polkavex project for production hosting

set -e

echo "🌉 Polkavex Deployment Script"
echo "=============================="

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if required tools are installed
check_dependencies() {
    print_status "Checking dependencies..."
    
    commands=("node" "npm" "git")
    for cmd in "${commands[@]}"; do
        if ! command -v $cmd &> /dev/null; then
            print_error "$cmd is not installed. Please install it first."
            exit 1
        fi
    done
    
    print_success "All dependencies are installed"
}

# Install all project dependencies
install_dependencies() {
    print_status "Installing project dependencies..."
    
    # Install root dependencies
    npm install
    
    # Install UI dependencies
    print_status "Installing UI dependencies..."
    cd ui && npm install && cd ..
    
    # Install relayer dependencies
    print_status "Installing relayer dependencies..."
    cd relayer && npm install && cd ..
    
    # Install ethereum dependencies
    print_status "Installing Ethereum dependencies..."
    cd ethereum && npm install && cd ..
    
    print_success "All dependencies installed successfully"
}

# Build all components
build_project() {
    print_status "Building project components..."
    
    # Build UI
    print_status "Building React UI..."
    cd ui && npm run build && cd ..
    
    # Build relayer
    print_status "Building relayer service..."
    cd relayer && npm run build && cd ..
    
    print_success "All components built successfully"
}

# Create production environment files
create_env_files() {
    print_status "Creating environment configuration..."
    
    # Create UI environment file
    cat > ui/.env.production << 'EOF'
REACT_APP_API_URL=https://your-relayer-domain.com
REACT_APP_ENVIRONMENT=production
REACT_APP_CHAIN_ID=1
REACT_APP_NETWORK=mainnet
EOF

    # Create relayer environment file
    cat > relayer/.env.production << 'EOF'
NODE_ENV=production
PORT=3002
CORS_ORIGIN=https://your-ui-domain.com
ETHEREUM_RPC_URL=https://mainnet.infura.io/v3/YOUR_INFURA_KEY
POLKADOT_WS_URL=wss://rpc.polkadot.io
GEMINI_API_KEY=your_gemini_api_key_here
ONEINCH_API_KEY=your_1inch_api_key_here
LOG_LEVEL=info
EOF

    print_warning "Please update the environment files with your actual values:"
    print_warning "- ui/.env.production"
    print_warning "- relayer/.env.production"
}

# Create Docker configurations
create_docker_config() {
    print_status "Creating Docker configuration..."
    
    # Create Dockerfile for UI
    cat > ui/Dockerfile << 'EOF'
FROM node:18-alpine AS builder

WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production

COPY . .
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/build /usr/share/nginx/html
COPY nginx.conf /etc/nginx/nginx.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
EOF

    # Create nginx config for UI
    cat > ui/nginx.conf << 'EOF'
events {}
http {
    include /etc/nginx/mime.types;
    default_type application/octet-stream;
    
    server {
        listen 80;
        root /usr/share/nginx/html;
        index index.html;
        
        location / {
            try_files $uri $uri/ /index.html;
        }
        
        location /static {
            expires 1y;
            add_header Cache-Control "public, immutable";
        }
    }
}
EOF

    # Create Dockerfile for relayer
    cat > relayer/Dockerfile << 'EOF'
FROM node:18-alpine

WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production

COPY . .
RUN npm run build

EXPOSE 3002
CMD ["npm", "start"]
EOF

    # Create docker-compose.yml
    cat > docker-compose.yml << 'EOF'
version: '3.8'

services:
  ui:
    build: ./ui
    ports:
      - "80:80"
    environment:
      - NODE_ENV=production
    depends_on:
      - relayer

  relayer:
    build: ./relayer
    ports:
      - "3002:3002"
    environment:
      - NODE_ENV=production
    env_file:
      - relayer/.env.production
    restart: unless-stopped

  redis:
    image: redis:alpine
    ports:
      - "6379:6379"
    restart: unless-stopped
EOF

    print_success "Docker configuration created"
}

# Create Vercel configuration
create_vercel_config() {
    print_status "Creating Vercel configuration..."
    
    cat > ui/vercel.json << 'EOF'
{
  "version": 2,
  "builds": [
    {
      "src": "package.json",
      "use": "@vercel/static-build",
      "config": {
        "distDir": "build"
      }
    }
  ],
  "routes": [
    {
      "src": "/static/(.*)",
      "headers": {
        "cache-control": "public, max-age=31536000, immutable"
      }
    },
    {
      "src": "/(.*)",
      "dest": "/index.html"
    }
  ],
  "env": {
    "REACT_APP_API_URL": "@api_url",
    "REACT_APP_ENVIRONMENT": "production"
  }
}
EOF

    print_success "Vercel configuration created"
}

# Create Railway configuration
create_railway_config() {
    print_status "Creating Railway configuration..."
    
    cat > relayer/railway.json << 'EOF'
{
  "$schema": "https://railway.app/railway.schema.json",
  "build": {
    "builder": "NIXPACKS"
  },
  "deploy": {
    "startCommand": "npm start",
    "healthcheckPath": "/health",
    "healthcheckTimeout": 100,
    "restartPolicyType": "ON_FAILURE",
    "restartPolicyMaxRetries": 10
  }
}
EOF

    cat > relayer/nixpacks.toml << 'EOF'
[phases.setup]
nixPkgs = ["nodejs", "npm"]

[phases.install]
cmds = ["npm ci"]

[phases.build]
cmds = ["npm run build"]

[start]
cmd = "npm start"
EOF

    print_success "Railway configuration created"
}

# Create deployment scripts
create_deploy_scripts() {
    print_status "Creating deployment scripts..."
    
    # Quick deploy script
    cat > deploy-quick.sh << 'EOF'
#!/bin/bash
# Quick deployment to free hosting services

echo "🚀 Quick Deploy to Free Services"
echo "================================"

# Build everything
npm run build

echo "✅ Built successfully!"
echo ""
echo "Next steps:"
echo "1. Push to GitHub: git add . && git commit -m 'Deploy' && git push"
echo "2. Frontend (Vercel):"
echo "   - Go to vercel.com"
echo "   - Import your GitHub repo"
echo "   - Set root directory to 'ui'"
echo "   - Deploy!"
echo ""
echo "3. Backend (Railway):"
echo "   - Go to railway.app"
echo "   - Import your GitHub repo"
echo "   - Set root directory to 'relayer'"
echo "   - Add environment variables"
echo "   - Deploy!"
echo ""
echo "4. Update CORS origins in relayer/.env.production"
EOF

    chmod +x deploy-quick.sh

    # VPS deploy script
    cat > deploy-vps.sh << 'EOF'
#!/bin/bash
# Deploy to VPS with Docker

echo "🚀 VPS Deployment with Docker"
echo "============================="

# Build and start services
docker-compose up -d --build

echo "✅ Services started!"
echo ""
echo "Access your app:"
echo "- Frontend: http://your-server-ip"
echo "- Backend API: http://your-server-ip:3002"
echo ""
echo "To check logs:"
echo "docker-compose logs -f"
EOF

    chmod +x deploy-vps.sh

    print_success "Deployment scripts created"
}

# Main deployment function
main() {
    echo ""
    print_status "Starting Polkavex deployment setup..."
    echo ""
    
    check_dependencies
    install_dependencies
    build_project
    create_env_files
    create_docker_config
    create_vercel_config
    create_railway_config
    create_deploy_scripts
    
    echo ""
    print_success "🎉 Deployment setup complete!"
    echo ""
    print_status "Choose your deployment method:"
    echo "1. Free hosting: ./deploy-quick.sh"
    echo "2. VPS hosting: ./deploy-vps.sh"
    echo "3. Manual deployment: See HOSTING_GUIDE.md"
    echo ""
    print_warning "Don't forget to:"
    print_warning "- Update environment variables"
    print_warning "- Configure your domains"
    print_warning "- Set up SSL certificates (for VPS)"
    echo ""
}

# Run main function
main "$@"
