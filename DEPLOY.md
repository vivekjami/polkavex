# 🚀 Polkavex Deployment Guide

## Quick Start (Free Hosting)

### Option 1: One-Click Deploy
```bash
./deploy-quick.sh
```

### Option 2: Manual Steps

#### 1. Build the project
```bash
npm install
cd ui && npm install && npm run build && cd ..
cd relayer && npm install && npm run build && cd ..
```

#### 2. Deploy Frontend (Vercel - Free)
1. Push your code to GitHub
2. Go to [vercel.com](https://vercel.com)
3. Import your GitHub repository
4. Set **Root Directory**: `ui`
5. Deploy!

#### 3. Deploy Backend (Railway - Free)
1. Go to [railway.app](https://railway.app)
2. Import your GitHub repository  
3. Set **Root Directory**: `relayer`
4. Add environment variables:
   ```
   NODE_ENV=production
   PORT=3002
   CORS_ORIGIN=https://your-vercel-app.vercel.app
   ETHEREUM_RPC_URL=https://mainnet.infura.io/v3/YOUR_KEY
   POLKADOT_WS_URL=wss://rpc.polkadot.io
   ```
5. Deploy!

#### 4. Update Configuration
- Update `ui/.env.production` with your Railway backend URL
- Update `relayer/.env.production` with your Vercel frontend URL

## Alternative Hosting Options

### Frontend Options
| Service | Cost | Features |
|---------|------|----------|
| **Vercel** | Free | CDN, Auto SSL, GitHub integration |
| **Netlify** | Free | Forms, Functions, Split testing |
| **GitHub Pages** | Free | Simple static hosting |
| **Firebase Hosting** | Free | Google integration |

### Backend Options
| Service | Cost | Features |
|---------|------|----------|
| **Railway** | Free 500h/month | Postgres, Redis, Auto SSL |
| **Render** | Free 750h/month | Auto SSL, GitHub integration |
| **Heroku** | $7/month | Add-ons ecosystem |
| **DigitalOcean** | $6/month | Full VPS control |

## VPS Deployment (Docker)

### Prerequisites
- Ubuntu 20.04+ server
- Docker & Docker Compose installed
- Domain name (optional)

### Deploy with Docker
```bash
# Clone your repo
git clone YOUR_REPO_URL
cd polkavex

# Create environment files
cp ui/.env.production.template ui/.env.production
cp relayer/.env.production.template relayer/.env.production

# Edit environment files with your values
nano ui/.env.production
nano relayer/.env.production

# Deploy with Docker
docker-compose up -d --build
```

### Access Your App
- Frontend: `http://your-server-ip`
- Backend: `http://your-server-ip:3002`

### SSL Setup (Optional)
```bash
# Install Nginx
sudo apt install nginx certbot python3-certbot-nginx

# Setup SSL with Let's Encrypt
sudo certbot --nginx -d your-domain.com
```

## Environment Variables

### UI (.env.production)
```env
REACT_APP_API_URL=https://your-backend-domain.com
REACT_APP_ENVIRONMENT=production
REACT_APP_CHAIN_ID=1
```

### Relayer (.env.production)
```env
NODE_ENV=production
PORT=3002
CORS_ORIGIN=https://your-frontend-domain.com
ETHEREUM_RPC_URL=https://mainnet.infura.io/v3/YOUR_KEY
POLKADOT_WS_URL=wss://rpc.polkadot.io
```

## API Keys Needed

1. **Infura/Alchemy** - Ethereum RPC access
   - Sign up at [infura.io](https://infura.io) or [alchemy.com](https://alchemy.com)
   - Create a new project
   - Copy the API key/URL

2. **1inch API** - DEX aggregation (optional)
   - Sign up at [1inch.io](https://1inch.io)
   - Get API key from dashboard

3. **Gemini API** - AI routing (optional)
   - Sign up at [ai.google.dev](https://ai.google.dev)
   - Create API key

## Monitoring & Logs

### Check Application Status
```bash
# Docker deployment
docker-compose logs -f

# Check individual services
docker-compose logs ui
docker-compose logs relayer
```

### Health Checks
- Frontend: `https://your-domain.com/health`
- Backend: `https://your-api-domain.com/health`

## Troubleshooting

### Common Issues

1. **CORS Errors**
   - Ensure `CORS_ORIGIN` in backend matches frontend URL
   - Don't include trailing slashes

2. **Build Failures**
   - Check Node.js version (use 18+)
   - Clear npm cache: `npm cache clean --force`

3. **Connection Issues**
   - Verify RPC URLs are accessible
   - Check firewall settings for VPS deployment

### Support
- Check logs for detailed error messages
- Verify all environment variables are set
- Test API endpoints individually

## Production Checklist

- [ ] Environment variables configured
- [ ] SSL certificates installed (for VPS)
- [ ] Database backups scheduled (if using database)
- [ ] Monitoring/alerting setup
- [ ] Custom domain configured
- [ ] CORS origins updated
- [ ] API rate limits configured
- [ ] Security headers enabled

---

**🎉 Your Polkavex cross-chain bridge is now live!**

For detailed configuration options, see `HOSTING_GUIDE.md`.
