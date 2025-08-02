const express = require('express');
const { ethers } = require('ethers');
const { ApiPromise, WsProvider } = require('@polkadot/api');
const { GoogleGenerativeAI } = require('@google/generative-ai');
const WebSocket = require('ws');
require('dotenv').config();

const app = express();
const PORT = process.env.PORT || 3002;

// Initialize AI service
const genAI = new GoogleGenerativeAI(process.env.GEMINI_API_KEY);
const model = genAI.getGenerativeModel({ model: process.env.GEMINI_MODEL || 'gemini-2.5-flash' });

// Blockchain connections
let ethProvider;
let polkadotApi;
let escrowContract;

// Event tracking
const activeSwaps = new Map();
const swapHistory = [];

app.use(express.json());

// Enhanced health check with blockchain status
app.get('/health', async (req, res) => {
    try {
        const ethStatus = ethProvider ? 'connected' : 'disconnected';
        const polkadotStatus = polkadotApi && polkadotApi.isConnected ? 'connected' : 'disconnected';
        
        res.json({ 
            status: 'ok', 
            timestamp: new Date().toISOString(),
            service: 'Polkavex Relayer',
            ethereum: ethStatus,
            polkadot: polkadotStatus,
            activeSwaps: activeSwaps.size,
            totalSwaps: swapHistory.length
        });
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

// Day 3: Enhanced swap initiation with bidirectional support
app.post('/initiate-swap', async (req, res) => {
    try {
        const { fromChain, toChain, asset, amount, maker, beneficiary, timelock } = req.body;
        
        if (!fromChain || !toChain || !asset || !amount || !maker || !beneficiary) {
            return res.status(400).json({ error: 'Missing required parameters' });
        }

        // Generate secret and hash for HTLC
        const secret = ethers.randomBytes(32);
        const secretHash = ethers.keccak256(secret);
        
        // Get AI routing suggestion
        const aiRoute = await getAiRouting(asset, amount);
        
        // Check for partial fill opportunity
        const partialFillInfo = await checkPartialFill(asset, amount);
        
        const swapId = `swap_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
        
        // Store swap details
        const swapData = {
            swapId,
            fromChain,
            toChain,
            asset,
            amount,
            maker,
            beneficiary,
            secretHash: secretHash.toString(),
            secret: secret.toString(),
            timelock: timelock || Math.floor(Date.now() / 1000) + 3600, // 1 hour default
            status: 'initiated',
            aiRoute,
            partialFillInfo,
            createdAt: new Date().toISOString()
        };
        
        activeSwaps.set(swapId, swapData);
        
        // Start the swap process
        await initiateSwapOnChain(swapData);
        
        res.json({
            success: true,
            swapId,
            secretHash: secretHash.toString(),
            aiRoute,
            partialFillInfo,
            message: 'Swap initiated successfully'
        });
        
    } catch (error) {
        console.error('Swap initiation error:', error);
        res.status(500).json({ error: error.message });
    }
});

// Day 3: AI routing with Gemini integration
app.post('/suggest-route', async (req, res) => {
    try {
        const { token, amount, fromChain, toChain } = req.body;
        
        const route = await getAiRouting(token, amount, fromChain, toChain);
        
        res.json({
            success: true,
            suggestion: route
        });
        
    } catch (error) {
        console.error('AI routing error:', error);
        res.status(500).json({ error: error.message });
    }
});

// Day 3: Enhanced AI routing with Gemini
async function getAiRouting(asset, amount, fromChain = 'ethereum', toChain = 'polkadot') {
    try {
        const assetInfo = detectAssetType(asset);
        
        const prompt = `As a DeFi routing expert, suggest the best Polkadot parachain for swapping ${assetInfo.type} token with amount ${amount}. Consider factors like:
        - Liquidity depth
        - Transaction fees  
        - Speed of execution
        - Yield opportunities
        - Cross-chain compatibility
        
        Focus on these parachains: Acala (DeFi hub), Moonbeam (EVM compatible), Astar (dApp platform), Hydration (AMM), Centrifuge (RWA).
        
        Provide a JSON response with: parachain, reason, estimatedGas.`;

        const result = await model.generateContent(prompt);
        const response = await result.response;
        const text = response.text();
        
        // Try to parse as JSON, fallback to structured response
        try {
            return JSON.parse(text);
        } catch {
            return {
                parachain: assetInfo.isStablecoin ? 'Acala' : 'Moonbeam',
                reason: text.substring(0, 200) + '...',
                estimatedGas: '0.1 DOT'
            };
        }
        
    } catch (error) {
        console.error('AI routing error:', error);
        // Fallback routing logic
        const assetInfo = detectAssetType(asset);
        
        if (assetInfo.isStablecoin) {
            return {
                parachain: 'Acala',
                reason: `Acala provides the best stablecoin liquidity for ${assetInfo.type} with integrated DeFi protocols`,
                estimatedGas: '0.05 DOT'
            };
        } else if (assetInfo.isNFT) {
            return {
                parachain: 'Moonbeam',
                reason: 'Moonbeam offers the most robust NFT infrastructure with EVM compatibility',
                estimatedGas: '0.2 DOT'
            };
        } else {
            return {
                parachain: 'Astar',
                reason: 'Astar provides optimal routing for general token swaps with low fees',
                estimatedGas: '0.1 DOT'
            };
        }
    }
}

// Day 3: Partial fill checking
async function checkPartialFill(asset, amount) {
    try {
        // Simulate liquidity checking (in production, this would query DEX APIs)
        const availableLiquidity = Math.floor(Math.random() * amount * 1.5); // Mock liquidity
        
        if (availableLiquidity < amount) {
            const fillPercentage = (availableLiquidity / amount * 100).toFixed(1);
            return {
                canPartialFill: true,
                availableAmount: availableLiquidity,
                fillPercentage: `${fillPercentage}%`,
                remainingAmount: amount - availableLiquidity,
                recommendation: 'Execute partial fill now, queue remainder'
            };
        }
        
        return {
            canPartialFill: false,
            availableAmount: amount,
            fillPercentage: '100%',
            remainingAmount: 0,
            recommendation: 'Full fill available'
        };
        
    } catch (error) {
        console.error('Partial fill check error:', error);
        return {
            canPartialFill: false,
            availableAmount: amount,
            fillPercentage: '100%',
            remainingAmount: 0,
            recommendation: 'Full fill assumed'
        };
    }
}

// Day 3: Initiate swap on blockchain
async function initiateSwapOnChain(swapData) {
    try {
        if (swapData.fromChain === 'ethereum') {
            // Create escrow on Ethereum
            await createEthereumEscrow(swapData);
        } else if (swapData.fromChain === 'polkadot') {
            // Create escrow on Polkadot
            await createPolkadotEscrow(swapData);
        }
        
        console.log(`Swap ${swapData.swapId} initiated on ${swapData.fromChain}`);
        
    } catch (error) {
        console.error('On-chain initiation error:', error);
        throw error;
    }
}

// Day 3: Create Ethereum escrow
async function createEthereumEscrow(swapData) {
    try {
        // This would call the Ethereum contract
        console.log(`Creating Ethereum escrow for swap ${swapData.swapId}`);
        
        // Mock transaction for demo
        const mockTx = {
            hash: `0x${Math.random().toString(16).substr(2, 64)}`,
            blockNumber: Math.floor(Math.random() * 1000000),
            gasUsed: '121000'
        };
        
        swapData.ethereumTx = mockTx;
        swapData.status = 'ethereum_created';
        
        return mockTx;
        
    } catch (error) {
        console.error('Ethereum escrow creation error:', error);
        throw error;
    }
}

// Day 3: Create Polkadot escrow
async function createPolkadotEscrow(swapData) {
    try {
        // This would call the Polkadot pallet
        console.log(`Creating Polkadot escrow for swap ${swapData.swapId}`);
        
        // Mock transaction for demo  
        const mockTx = {
            hash: `0x${Math.random().toString(16).substr(2, 64)}`,
            blockNumber: Math.floor(Math.random() * 1000000),
            weight: '500000000'
        };
        
        swapData.polkadotTx = mockTx;
        swapData.status = 'polkadot_created';
        
        return mockTx;
        
    } catch (error) {
        console.error('Polkadot escrow creation error:', error);
        throw error;
    }
}

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

// Fallback AI routing function
async function fallbackAiRouting(asset) {
    const assetInfo = detectAssetType(asset);
    
    if (assetInfo.isStablecoin) {
        return {
            parachain: 'Acala',
            reason: `Acala provides the best stablecoin liquidity for ${assetInfo.type} with integrated DeFi protocols`,
            estimatedGas: '0.05 DOT'
        };
    } else if (assetInfo.isNFT) {
        return {
            parachain: 'Moonbeam',
            reason: 'Moonbeam offers the most robust NFT infrastructure with EVM compatibility',
            estimatedGas: '0.2 DOT'
        };
    } else {
        return {
            parachain: 'Astar',
            reason: 'Astar provides optimal routing for general token swaps with low fees',
            estimatedGas: '0.1 DOT'
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
            ethProvider = provider;
        } else {
            console.log('⚠️  Ethereum RPC URL not configured');
        }

        // Test Polkadot connection
        if (process.env.POLKADOT_WS_URL) {
            const provider = new WsProvider(process.env.POLKADOT_WS_URL);
            const api = await ApiPromise.create({ provider });
            const chain = await api.rpc.system.chain();
            console.log(`✅ Polkadot connected: ${chain}`);
            polkadotApi = api;
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
