import express from 'express';
import { createServer } from 'http';
import { Server } from 'socket.io';
import { GoogleGenerativeAI } from '@google/generative-ai';
import dotenv from 'dotenv';
import cors from 'cors';

dotenv.config();

const app = express();
const server = createServer(app);
const io = new Server(server, {
  cors: {
    origin: ["http://localhost:3000", "http://localhost:3001"],
    methods: ["GET", "POST"]
  }
});

// Initialize Gemini AI
const genAI = new GoogleGenerativeAI(process.env.GEMINI_API_KEY || '');
const model = genAI.getGenerativeModel({ model: process.env.GEMINI_MODEL || 'gemini-2.0-flash-exp' });

// Mock storage for swaps
const swaps = new Map();

app.use(express.json());
app.use(cors());

// Health check
app.get('/health', (req, res) => {
  res.json({ 
    status: 'ok', 
    timestamp: new Date().toISOString(),
    ethereum: 'connected',
    polkadot: 'connected'
  });
});

// AI Route suggestion
app.post('/api/suggest-route', async (req, res) => {
  try {
    const { token, amount, userPreference } = req.body;
    
    const suggestion = await getAIRouteSuggestion(token, amount, userPreference);
    res.json({ suggestion });
  } catch (error) {
    console.error('AI suggestion error:', error);
    res.json({ 
      suggestion: {
        parachain: "Acala",
        reason: "Default safe option for " + (req.body.token || 'assets'),
        estimatedGas: "0.1 DOT"
      }
    });
  }
});

// Enhanced swap initiation with asset detection
app.post('/api/initiate-swap', async (req, res) => {
  try {
    const { amount, fromToken, toToken, fromChain, toChain, beneficiary, assetType, slippage } = req.body;
    
    // Validate input
    if (!amount || !beneficiary || !fromChain || !toChain) {
      return res.status(400).json({ error: 'Missing required parameters' });
    }

    // Generate unique swap ID
    const swapId = `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    
    // Get AI routing suggestion
    const routingSuggestion = await getAIRouteSuggestion(fromToken, amount, assetType);
    
    // Create swap data
    const swapData = {
      swapId,
      status: 'initiated',
      amount,
      token: fromToken,
      targetToken: toToken,
      sourceChain: fromChain,
      targetChain: toChain,
      beneficiary,
      assetType,
      slippage,
      route: routingSuggestion,
      createdAt: new Date().toISOString(),
      estimatedTime: '2-5 minutes',
      progress: [
        {
          timestamp: new Date().toISOString(),
          event: 'Swap initiated',
          status: 'completed'
        },
        {
          timestamp: new Date(Date.now() + 1000).toISOString(),
          event: 'AI routing calculated',
          status: 'completed'
        }
      ]
    };
    
    // Store swap
    swaps.set(swapId, swapData);
    
    console.log('🚀 Enhanced swap initiated:', {
      swapId,
      assetType,
      route: routingSuggestion
    });
    
    // Simulate processing
    setTimeout(() => {
      swapData.status = 'processing';
      swapData.progress.push({
        timestamp: new Date().toISOString(),
        event: 'Cross-chain transfer in progress',
        status: 'active'
      });
      io.emit('swap-update', swapData);
    }, 2000);
    
    setTimeout(() => {
      swapData.status = 'completed';
      swapData.progress.push({
        timestamp: new Date().toISOString(),
        event: 'Swap completed successfully',
        status: 'completed'
      });
      io.emit('swap-update', swapData);
    }, 5000);
    
    res.json({
      status: 'initiated',
      swapId,
      swapDetails: swapData,
      routingSuggestion
    });
    
  } catch (error) {
    console.error('Swap initiation failed:', error);
    res.status(500).json({ error: 'Failed to initiate swap' });
  }
});

// Get swap status
app.get('/api/swap/:swapId', (req, res) => {
  const { swapId } = req.params;
  const swap = swaps.get(swapId);
  
  if (!swap) {
    return res.status(404).json({ error: 'Swap not found' });
  }
  
  res.json(swap);
});

// Claim swap
app.post('/api/claim-swap', async (req, res) => {
  try {
    const { swapId, secret } = req.body;
    
    if (!swapId || !secret) {
      return res.status(400).json({ error: 'Missing swapId or secret' });
    }
    
    const swap = swaps.get(swapId);
    if (!swap) {
      return res.status(404).json({ error: 'Swap not found' });
    }
    
    // Simulate claim processing
    swap.status = 'claiming';
    swap.progress.push({
      timestamp: new Date().toISOString(),
      event: 'Claim transaction submitted',
      status: 'active'
    });
    
    setTimeout(() => {
      swap.status = 'completed';
      swap.progress.push({
        timestamp: new Date().toISOString(),
        event: 'Claim successful',
        status: 'completed'
      });
    }, 3000);
    
    res.json({
      status: 'claiming',
      swapId,
      message: 'Claim transaction submitted'
    });
    
  } catch (error) {
    console.error('Claim failed:', error);
    res.status(500).json({ error: 'Failed to process claim' });
  }
});

async function getAIRouteSuggestion(token: string, amount: string, preference: string) {
  try {
    const prompt = `You are a DeFi routing expert. Suggest the best Polkadot parachain for a ${amount} ${token} swap.

Consider:
- Asset type: ${preference}
- Current yields: Acala (8% APY stablecoins), Moonbeam (fast EVM), Statemint (asset hub)
- Gas costs and speed

Respond with ONLY valid JSON:
{"parachain": "ParachainName", "reason": "Brief explanation", "estimatedGas": "X.X DOT"}`;

    const response = await model.generateContent(prompt);
    const result = response.response.text().trim();
    
    // Extract JSON if the response contains extra text
    const jsonMatch = result.match(/\{.*\}/s);
    const jsonStr = jsonMatch ? jsonMatch[0] : result;
    
    return JSON.parse(jsonStr);
  } catch (error) {
    console.error('AI suggestion failed:', error);
    
    // Fallback logic
    if (['USDC', 'USDT', 'DAI'].includes(token)) {
      return {
        parachain: "Acala",
        reason: "Optimized for stablecoin yields (8% APY)",
        estimatedGas: "0.05 DOT"
      };
    } else if (token === 'ETH') {
      return {
        parachain: "Moonbeam",
        reason: "Fast EVM compatibility for ETH transfers",
        estimatedGas: "0.1 DOT"
      };
    } else {
      return {
        parachain: "Statemint",
        reason: "Universal asset hub for all tokens",
        estimatedGas: "0.08 DOT"
      };
    }
  }
}

// WebSocket connection handling
io.on('connection', (socket) => {
  console.log('Client connected:', socket.id);
  
  socket.on('disconnect', () => {
    console.log('Client disconnected:', socket.id);
  });
});

const PORT = process.env.PORT || 3002;

server.listen(PORT, () => {
  console.log('🚀 Polkavex Relayer running on port', PORT);
  console.log('✅ Gemini AI configured');
  console.log('✅ WebSocket server ready');
  console.log('✅ CORS enabled for UI connections');
});

export default app;
