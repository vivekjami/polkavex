#!/bin/bash

# 🚀 Quick Deploy Script for Free Hosting
# This script helps you deploy Polkavex to free hosting services

set -e

echo "🌉 Polkavex Quick Deploy"
echo "======================="
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_info() {
    echo -e "${YELLOW}[INFO]${NC} $1"
}

# Step 1: Install dependencies and build
print_step "Installing dependencies and building project..."
npm install
cd ui && npm install && npm run build && cd ..
cd relayer && npm install && npm run build && cd ..
print_success "Build completed!"

# Step 2: Create environment files
print_step "Creating environment templates..."
cp ui/.env.production.template ui/.env.production
cp relayer/.env.production.template relayer/.env.production
print_success "Environment files created!"

# Step 3: Initialize git if not already done
if [ ! -d ".git" ]; then
    print_step "Initializing git repository..."
    git init
    git add .
    git commit -m "Initial commit: Polkavex cross-chain bridge"
    print_success "Git repository initialized!"
else
    print_info "Git repository already exists"
fi

echo ""
echo "🎉 Build completed successfully!"
echo ""
echo "Next steps for deployment:"
echo ""
echo "1. 📱 Frontend Deployment (Choose one):"
echo "   • Vercel (Recommended):"
echo "     - Push to GitHub: git remote add origin YOUR_REPO_URL && git push -u origin main"
echo "     - Go to vercel.com and import your repo"
echo "     - Set root directory to 'ui' and deploy"
echo ""
echo "   • Netlify:"
echo "     - Drag & drop the 'ui/build' folder to netlify.com/drop"
echo "     - Or connect your GitHub repo with root directory 'ui'"
echo ""
echo "2. 🔧 Backend Deployment (Choose one):"
echo "   • Railway (Recommended):"
echo "     - Go to railway.app and import your GitHub repo"
echo "     - Set root directory to 'relayer'"
echo "     - Add environment variables from relayer/.env.production"
echo ""
echo "   • Render:"
echo "     - Go to render.com and create a new web service"
echo "     - Connect your GitHub repo with root directory 'relayer'"
echo "     - Set build command: 'npm install && npm run build'"
echo "     - Set start command: 'npm start'"
echo ""
echo "3. 🔐 Environment Configuration:"
echo "   • Update ui/.env.production with your backend URL"
echo "   • Update relayer/.env.production with your API keys:"
echo "     - Infura/Alchemy for Ethereum RPC"
echo "     - 1inch API key"
echo "     - Gemini API key (optional)"
echo ""
echo "4. 🌐 Domain Setup:"
echo "   • Update CORS_ORIGIN in relayer config with your frontend URL"
echo "   • Update REACT_APP_API_URL in UI config with your backend URL"
echo ""
echo "💡 Pro Tips:"
echo "   • Use environment variables in hosting platforms (don't commit secrets)"
echo "   • Enable automatic deployments from your main branch"
echo "   • Set up custom domains for professional URLs"
echo ""
echo "🔗 Useful Links:"
echo "   • Vercel: https://vercel.com"
echo "   • Netlify: https://netlify.com"
echo "   • Railway: https://railway.app"
echo "   • Render: https://render.com"
echo ""
echo "Need help? Check the HOSTING_GUIDE.md for detailed instructions!"
