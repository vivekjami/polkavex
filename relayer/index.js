const express = require('express');
const { ethers } = require('ethers');
const { ApiPromise, WsProvider } = require('@polkadot/api');
const { GoogleGenerativeAI } = require('@google/generative-ai');
const WebSocket = require('ws');
const OneInchService = require('./services/oneinch');
require('dotenv').config();

const app = express();
const PORT = process.env.PORT || 3002;

// Initialize AI service
const genAI = new GoogleGenerativeAI(process.env.GEMINI_API_KEY);
const model = genAI.getGenerativeModel({ model: process.env.GEMINI_MODEL || 'gemini-2.5-flash' });

// Initialize 1inch service
const oneInchService = new OneInchService(process.env.ONEINCH_API_KEY, 1); // Ethereum mainnet

// Blockchain connections
let ethProvider;
let polkadotApi;
let escrowContract;

// Event tracking
const activeSwaps = new Map();
const swapHistory = [];

app.use(express.json());

// Enable CORS for frontend
app.use((req, res, next) => {
  res.header('Access-Control-Allow-Origin', '*');
  res.header('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS');
  res.header('Access-Control-Allow-Headers', 'Origin, X-Requested-With, Content-Type, Accept, Authorization');
  if (req.method === 'OPTIONS') {
    res.sendStatus(200);
  } else {
    next();
  }
});

// Enhanced health check with blockchain status
app.get('/health', async (req, res) => {
    try {
        const ethStatus = ethProvider ? 'connected' : 'disconnected';
        const polkadotStatus = polkadotApi && polkadotApi.isConnected ? 'connected' : 'disconnected';
        
        // Check 1inch API status
        const oneInchStatus = await oneInchService.healthCheck();
        
        res.json({ 
            status: 'ok', 
            timestamp: new Date().toISOString(),
            service: 'Polkavex Relayer',
            ethereum: ethStatus,
            polkadot: polkadotStatus,
            oneinch: oneInchStatus.status,
            oneInchDetails: oneInchStatus,
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

// Day 3: Enhanced AI routing with Gemini and 1inch integration
async function getAiRouting(asset, amount, fromChain = 'ethereum', toChain = 'polkadot') {
    try {
        const assetInfo = detectAssetType(asset);
        
        // Get 1inch quote for comparison if dealing with Ethereum
        let oneInchData = null;
        if (fromChain === 'ethereum' && process.env.ONEINCH_API_KEY) {
            try {
                // For demo purposes, use USDC as destination for 1inch comparison
                const usdcAddress = '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48';
                if (asset !== usdcAddress) {
                    const quote = await oneInchService.getSwapQuote(asset, usdcAddress, amount);
                    if (quote.success) {
                        oneInchData = {
                            available: true,
                            estimatedOutput: quote.data.dstAmount,
                            gasCost: quote.estimatedGas,
                            protocols: quote.protocols.length
                        };
                    }
                }
            } catch (error) {
                console.log('1inch data unavailable for routing decision');
            }
        }

        const prompt = `As a DeFi routing expert, suggest the best Polkadot parachain for swapping ${assetInfo.type} token with amount ${amount}. 

Available data:
- Asset type: ${assetInfo.type}
- Is stablecoin: ${assetInfo.isStablecoin}
- Is NFT: ${assetInfo.isNFT}
- 1inch availability: ${oneInchData ? 'Available with ' + oneInchData.protocols + ' protocols' : 'Not available'}

Consider factors like:
- Liquidity depth
- Transaction fees  
- Speed of execution
- Yield opportunities
- Cross-chain compatibility
- 1inch vs parachain benefits

Focus on these parachains: Acala (DeFi hub), Moonbeam (EVM compatible), Astar (dApp platform), Hydration (AMM), Centrifuge (RWA).
        
Provide a JSON response with: parachain, reason, estimatedGas, oneInchComparison.`;

        const result = await model.generateContent(prompt);
        const response = await result.response;
        const text = response.text();
        
        // Try to parse as JSON, fallback to structured response
        try {
            const parsed = JSON.parse(text);
            return {
                ...parsed,
                oneInchData,
                confidence: 0.85,
                source: 'AI + 1inch'
            };
        } catch {
            return {
                parachain: assetInfo.isStablecoin ? 'Acala' : 'Moonbeam',
                reason: text.substring(0, 200) + '...',
                estimatedGas: '0.1 DOT',
                oneInchData,
                confidence: 0.75,
                source: 'AI fallback'
            };
        }
        
    } catch (error) {
        console.error('AI routing error:', error);
        return await fallbackAiRouting(asset);
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

// Fallback AI routing function with 1inch awareness
async function fallbackAiRouting(asset) {
    const assetInfo = detectAssetType(asset);
    
    // Check 1inch availability for context
    let oneInchAvailable = false;
    if (process.env.ONEINCH_API_KEY) {
        try {
            const healthCheck = await oneInchService.healthCheck();
            oneInchAvailable = healthCheck.status === 'connected';
        } catch (error) {
            oneInchAvailable = false;
        }
    }
    
    if (assetInfo.isStablecoin) {
        return {
            parachain: 'Acala',
            reason: `Acala provides optimal stablecoin liquidity for ${assetInfo.type}. ${oneInchAvailable ? '1inch available as alternative.' : 'Cross-chain preferred for yield opportunities.'}`,
            estimatedGas: '0.05 DOT',
            oneInchAlternative: oneInchAvailable,
            source: 'fallback + 1inch check'
        };
    } else if (assetInfo.isNFT) {
        return {
            parachain: 'Moonbeam',
            reason: 'Moonbeam offers the most robust NFT infrastructure with EVM compatibility',
            estimatedGas: '0.2 DOT',
            oneInchAlternative: false, // 1inch doesn't handle NFTs well
            source: 'fallback'
        };
    } else {
        return {
            parachain: 'Astar',
            reason: `Astar provides optimal routing for general token swaps. ${oneInchAvailable ? 'Consider 1inch for immediate execution.' : ''}`,
            estimatedGas: '0.1 DOT',
            oneInchAlternative: oneInchAvailable,
            source: 'fallback + 1inch check'
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

// 1inch Integration: Get price quote
app.post('/api/1inch/quote', async (req, res) => {
    try {
        const { fromToken, toToken, amount, slippage } = req.body;
        
        if (!fromToken || !toToken || !amount) {
            return res.status(400).json({ error: 'Missing required parameters: fromToken, toToken, amount' });
        }

        const quote = await oneInchService.getSwapQuote(fromToken, toToken, amount, slippage);
        
        if (quote.success) {
            res.json({
                success: true,
                quote: quote.data,
                estimatedGas: quote.estimatedGas,
                minReturn: quote.minReturn,
                protocols: quote.protocols,
                source: '1inch'
            });
        } else {
            res.json({
                success: false,
                error: quote.error,
                fallback: 'Using cross-chain routing instead'
            });
        }
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

// 1inch Integration: Get optimized route (1inch vs cross-chain)
app.post('/api/1inch/optimize-route', async (req, res) => {
    try {
        const { fromToken, toToken, amount, userPreference } = req.body;
        
        if (!fromToken || !toToken || !amount) {
            return res.status(400).json({ error: 'Missing required parameters' });
        }

        const optimization = await oneInchService.getOptimizedRoute(
            fromToken, 
            toToken, 
            amount, 
            userPreference || 'best_price'
        );
        
        res.json(optimization);
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

// 1inch Integration: Create Fusion+ order
app.post('/api/1inch/fusion-order', async (req, res) => {
    try {
        const orderData = req.body;
        
        const fusionOrder = await oneInchService.createFusionOrder(orderData);
        
        if (fusionOrder.success) {
            res.json({
                success: true,
                order: fusionOrder.order,
                orderHash: fusionOrder.orderHash,
                message: 'Fusion+ order ready for cross-chain execution'
            });
        } else {
            res.status(400).json({
                success: false,
                error: fusionOrder.error
            });
        }
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

// 1inch Integration: Check token liquidity
app.post('/api/1inch/liquidity', async (req, res) => {
    try {
        const { fromToken, toToken, amount } = req.body;
        
        const liquidity = await oneInchService.checkLiquidity(fromToken, toToken, amount);
        res.json(liquidity);
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

app.listen(PORT, async () => {
    console.log(`🚀 Polkavex Relayer running on port ${PORT}`);
    await initializeConnections();
});

module.exports = { app, getAiRouting, detectAssetType, resolveEscrow };
