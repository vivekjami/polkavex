import express from 'express';
import { createServer } from 'http';
import { Server } from 'socket.io';
import { ethers } from 'ethers';
import { ApiPromise, WsProvider } from '@polkadot/api';
import OpenAI from 'openai';
import dotenv from 'dotenv';

dotenv.config();

const app = express();
const server = createServer(app);
const io = new Server(server, {
  cors: {
    origin: "http://localhost:3000",
    methods: ["GET", "POST"]
  }
});

// Initialize OpenAI
const openai = new OpenAI({
  apiKey: process.env.OPENAI_API_KEY || ''
});

// Initialize providers
let ethProvider: ethers.Provider;
let polkadotApi: ApiPromise;

class PolkavexRelayer {
  constructor() {
    this.initializeProviders();
    this.setupRoutes();
    this.setupWebSocket();
  }

  async initializeProviders() {
    try {
      // Ethereum provider
      ethProvider = new ethers.JsonRpcProvider(process.env.ETHEREUM_RPC_URL);
      console.log('✅ Ethereum provider connected');

      // Polkadot provider  
      const wsProvider = new WsProvider(process.env.POLKADOT_WS_URL || 'wss://westend-rpc.polkadot.io');
      polkadotApi = await ApiPromise.create({ provider: wsProvider });
      console.log('✅ Polkadot API connected');
    } catch (error) {
      console.error('❌ Provider initialization failed:', error);
    }
  }

  setupRoutes() {
    app.use(express.json());

    // Health check
    app.get('/health', (req, res) => {
      res.json({ 
        status: 'ok', 
        timestamp: new Date().toISOString(),
        ethereum: ethProvider ? 'connected' : 'disconnected',
        polkadot: polkadotApi ? 'connected' : 'disconnected'
      });
    });

    // AI Route suggestion
    app.post('/api/suggest-route', async (req, res) => {
      try {
        const { token, amount, userPreference } = req.body;
        
        const suggestion = await this.getAIRouteSuggestion(token, amount, userPreference);
        res.json({ suggestion });
      } catch (error) {
        res.status(500).json({ error: 'Failed to get route suggestion' });
      }
    });

    // Initiate swap
    app.post('/api/initiate-swap', async (req, res) => {
      try {
        const { secretHash, amount, token, timelock } = req.body;
        
        // This will be implemented in Day 3
        res.json({ 
          status: 'initiated',
          swapId: secretHash,
          message: 'Swap initiation logic will be implemented in Day 3'
        });
      } catch (error) {
        res.status(500).json({ error: 'Failed to initiate swap' });
      }
    });
  }

  setupWebSocket() {
    io.on('connection', (socket) => {
      console.log('Client connected:', socket.id);

      socket.on('subscribe-swap', (swapId) => {
        socket.join(`swap-${swapId}`);
        console.log(`Client ${socket.id} subscribed to swap ${swapId}`);
      });

      socket.on('disconnect', () => {
        console.log('Client disconnected:', socket.id);
      });
    });
  }

  async getAIRouteSuggestion(token: string, amount: string, preference: string) {
    try {
      const prompt = `As a DeFi routing expert, suggest the best Polkadot parachain for a ${amount} ${token} swap.
      User preference: ${preference}
      
      Consider:
      - Acala: 8% APY for stablecoin yields
      - Moonbeam: Fast EVM compatibility
      - Statemint: Asset hub for NFTs/assets
      
      Respond with JSON: {"parachain": "name", "reason": "explanation", "estimatedGas": "estimate"}`;

      const response = await openai.chat.completions.create({
        model: 'gpt-3.5-turbo',
        messages: [{ role: 'user', content: prompt }],
        max_tokens: 200
      });

      return JSON.parse(response.choices[0].message.content || '{}');
    } catch (error) {
      console.error('AI suggestion failed:', error);
      return {
        parachain: 'Statemint',
        reason: 'Default safe option',
        estimatedGas: '0.1 DOT'
      };
    }
  }

  // Monitor blockchain events (Day 3 implementation)
  async startEventMonitoring() {
    console.log('🔍 Event monitoring will be implemented in Day 3');
  }
}

const relayer = new PolkavexRelayer();

const PORT = process.env.PORT || 3001;
server.listen(PORT, () => {
  console.log(`🚀 Polkavex Relayer running on port ${PORT}`);
  console.log(`📊 Health check: http://localhost:${PORT}/health`);
});

export default relayer;
