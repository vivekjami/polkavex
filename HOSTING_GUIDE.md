# 🚀 Polkavex Hosting Guide

## Overview
This guide helps you deploy the complete Polkavex cross-chain bridge for demo purposes.

## Architecture
- **Frontend (UI)**: React application
- **Backend (Relayer)**: Node.js/Express API server
- **Smart Contracts**: Ethereum contracts (already deployed)
- **Polkadot Runtime**: Local development node

## 🌐 Hosting Options

### Option 1: Free Hosting (Recommended for Demo)

#### Frontend - Vercel (Free)
1. **Push to GitHub**:
   ```bash
   git add .
   git commit -m "Add Polkavex demo"
   git push origin main
   ```

2. **Deploy to Vercel**:
   - Visit [vercel.com](https://vercel.com)
   - Connect your GitHub repo
   - Set build directory: `ui`
   - Set build command: `npm run build`
   - Set output directory: `build`

#### Backend - Railway/Render (Free)
- **Railway**: Railway.app (500 hours free/month)
- **Render**: render.com (750 hours free/month)

#### Alternative: Netlify + Heroku
- **Frontend**: Netlify (100GB bandwidth free)
- **Backend**: Heroku (Free dyno sleeping)

### Option 2: Single VPS (Cost-effective)

#### DigitalOcean Droplet ($6/month)
- 1GB RAM, 1 vCPU, 25GB SSD
- Ubuntu 22.04 LTS
- Nginx reverse proxy
- PM2 for process management

#### Linode/Vultr ($5-10/month)
- Similar specs to DigitalOcean
- Good performance for demo

### Option 3: Cloud Platforms

#### AWS (Free Tier)
- **Frontend**: S3 + CloudFront
- **Backend**: EC2 t2.micro
- **Database**: RDS free tier (if needed)

#### Google Cloud (Free Tier)
- **Frontend**: Firebase Hosting
- **Backend**: Cloud Run
- **Functions**: Cloud Functions

## 🔧 Quick Deploy Scripts

Let me create automated deployment scripts for you...
