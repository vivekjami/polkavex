const express = require('express');
const { ethers } = require('ethers');
const { ApiPromise, WsProvider } = require('@polkadot/api');
require('dotenv').config();

const app = express();
const PORT = process.env.PORT || 3001;

app.use(express.json());

// Basic health check endpoint
app.get('/health', (req, res) => {
    res.json({ 
        status: 'ok', 
        timestamp: new Date().toISOString(),
        service: 'Polkavex Relayer'
    });
});

// Placeholder for swap initiation
app.post('/initiate-swap', async (req, res) => {
    try {
        // TODO: Implement swap logic
        res.json({ 
            message: 'Swap initiation endpoint - coming soon!',
            data: req.body 
        });
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

// Placeholder for AI routing
app.post('/suggest-route', async (req, res) => {
    try {
        // TODO: Implement AI routing with OpenAI
        res.json({ 
            message: 'AI routing endpoint - coming soon!',
            suggested_parachain: 'acala',
            reason: 'Higher yields available'
        });
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

// Asset type detection helper
function detectAssetType(asset) {
    if (!asset || asset === '0x0000000000000000000000000000000000000000') {
        return { type: 'ETH', isNFT: false, isStablecoin: false };
    }
    
    // Known stablecoin addresses (Sepolia testnet)
    const stablecoins = {
        '0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238': 'USDC',
        '0x7169D38820dfd117C3FA1f22a697dBA58d90BA06': 'USDT',
        '0xFF34B3d4Aee8ddCd6F9AFFFB6Fe49bD371b8a357': 'DAI'
    };
    
    if (stablecoins[asset]) {
        return { type: stablecoins[asset], isNFT: false, isStablecoin: true };
    }
    
    return { type: 'ERC20', isNFT: false, isStablecoin: false };
}

// Enhanced swap resolution with NFT support
async function resolveEscrow(escrowId, secret, assetInfo) {
    try {
        console.log(`Resolving escrow ${escrowId} with asset type:`, assetInfo);
        
        if (assetInfo.isNFT) {
            console.log('Handling NFT transfer via contract call');
            // Handle tokenId transfer via contract call
            // This would integrate with the ERC721 transfer logic
            return { success: true, type: 'NFT', tokenId: assetInfo.tokenId };
        } else if (assetInfo.isStablecoin) {
            console.log(`Handling stablecoin transfer: ${assetInfo.type}`);
            // Enhanced stablecoin handling with proper decimals
            return { success: true, type: 'STABLECOIN', token: assetInfo.type };
        } else {
            console.log('Handling standard ERC20/ETH transfer');
            return { success: true, type: 'STANDARD' };
        }
    } catch (error) {
        console.error('Error resolving escrow:', error);
        throw error;
    }
}

// Enhanced AI routing with asset-specific suggestions
async function getAiRouting(asset, amount) {
    const assetInfo = detectAssetType(asset);
    
    // Asset-specific routing logic
    if (assetInfo.isStablecoin) {
        return {
            parachain: 'Acala',
            reason: `Optimized stablecoin liquidity for ${assetInfo.type}`,
            estimatedTime: '2-3 minutes',
            confidence: 0.95
        };
    } else if (assetInfo.isNFT) {
        return {
            parachain: 'Moonbeam',
            reason: 'Best NFT infrastructure and EVM compatibility',
            estimatedTime: '3-5 minutes',
            confidence: 0.88
        };
    } else {
        return {
            parachain: 'Astar',
            reason: 'Optimal for general ERC20 transfers',
            estimatedTime: '1-2 minutes',
            confidence: 0.92
        };
    }
}

async function initializeConnections() {
    try {
        console.log('Initializing blockchain connections...');
        
        // Test Ethereum connection
        if (process.env.ETHEREUM_RPC_URL) {
            const provider = new ethers.JsonRpcProvider(process.env.ETHEREUM_RPC_URL);
            const network = await provider.getNetwork();
            console.log(`✅ Ethereum connected: ${network.name} (${network.chainId})`);
        } else {
            console.log('⚠️  Ethereum RPC URL not configured');
        }

        // Test Polkadot connection
        if (process.env.POLKADOT_WS_URL) {
            const provider = new WsProvider(process.env.POLKADOT_WS_URL);
            const api = await ApiPromise.create({ provider });
            const chain = await api.rpc.system.chain();
            console.log(`✅ Polkadot connected: ${chain}`);
            await api.disconnect();
        } else {
            console.log('⚠️  Polkadot WS URL not configured');
        }

        console.log('Blockchain connections initialized');
    } catch (error) {
        console.error('Failed to initialize connections:', error.message);
    }
}

app.listen(PORT, async () => {
    console.log(`🚀 Polkavex Relayer running on port ${PORT}`);
    await initializeConnections();
});

module.exports = { app, getAiRouting, detectAssetType, resolveEscrow };
