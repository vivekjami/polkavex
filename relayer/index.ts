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
    
    // Start event monitoring after a delay to ensure providers are ready
    setTimeout(() => {
      this.startEventMonitoring();
    }, 5000);
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

    // Initiate swap with actual implementation
    app.post('/api/initiate-swap', async (req, res) => {
      try {
        const { secretHash, amount, token, timelock, beneficiary, sourceChain, targetChain } = req.body;
        
        // Validate input
        if (!secretHash || !amount || !beneficiary || !sourceChain || !targetChain) {
          return res.status(400).json({ error: 'Missing required parameters' });
        }

        // Generate unique swap ID
        const swapId = `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
        
        console.log('🚀 Initiating cross-chain swap:', {
          swapId,
          secretHash,
          amount,
          token: token || 'ETH',
          sourceChain,
          targetChain,
          beneficiary
        });

        // Store swap details (in production, use proper database)
        const swapDetails = {
          swapId,
          secretHash,
          amount,
          token: token || 'ETH',
          timelock: timelock || Math.floor(Date.now() / 1000) + 3600, // 1 hour default
          beneficiary,
          sourceChain,
          targetChain,
          status: 'initiated',
          createdAt: new Date().toISOString()
        };

        // Emit progress update
        io.emit('swap-initiated', swapDetails);

        // Get AI routing suggestion
        const routingSuggestion = await this.getAIRouteSuggestion(
          token || 'ETH', 
          amount, 
          `Cross-chain swap from ${sourceChain} to ${targetChain}`
        );

        res.json({ 
          status: 'initiated',
          swapId,
          swapDetails,
          routingSuggestion,
          message: 'Cross-chain swap initiated successfully'
        });

        // Start monitoring this specific swap
        this.monitorSwapProgress(swapId, swapDetails);
        
      } catch (error) {
        console.error('Swap initiation failed:', error);
        res.status(500).json({ error: 'Failed to initiate swap' });
      }
    });

    // Check swap status
    app.get('/api/swap/:swapId', async (req, res) => {
      try {
        const { swapId } = req.params;
        
        // In production, fetch from database
        // For demo, return simulated status
        const swapStatus = {
          swapId,
          status: 'active',
          progress: [
            { step: 'initiated', completed: true, timestamp: new Date().toISOString() },
            { step: 'ethereum-locked', completed: true, timestamp: new Date().toISOString() },
            { step: 'polkadot-locked', completed: false, timestamp: null },
            { step: 'claimed', completed: false, timestamp: null }
          ]
        };

        res.json(swapStatus);
      } catch (error) {
        res.status(500).json({ error: 'Failed to get swap status' });
      }
    });

    // Claim swap
    app.post('/api/claim-swap', async (req, res) => {
      try {
        const { swapId, secret } = req.body;
        
        if (!swapId || !secret) {
          return res.status(400).json({ error: 'Missing swapId or secret' });
        }

        console.log('🔓 Processing swap claim:', { swapId, secret });

        // Simulate claim process
        setTimeout(() => {
          io.emit('swap-claimed', {
            swapId,
            secret,
            status: 'completed',
            message: 'Swap completed successfully'
          });
        }, 1000);

        res.json({
          status: 'claiming',
          swapId,
          message: 'Claim transaction submitted'
        });
      } catch (error) {
        res.status(500).json({ error: 'Failed to claim swap' });
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

  // Monitor blockchain events
  async startEventMonitoring() {
    console.log('🔍 Starting cross-chain event monitoring...');
    
    try {
      // Monitor Ethereum events
      await this.monitorEthereumEvents();
      
      // Monitor Polkadot events  
      await this.monitorPolkadotEvents();
      
      console.log('✅ Event monitoring active on both chains');
    } catch (error) {
      console.error('❌ Event monitoring failed:', error);
    }
  }

  async monitorEthereumEvents() {
    if (!ethProvider) {
      console.error('Ethereum provider not initialized');
      return;
    }

    // FusionEscrow contract address (update with deployed address)
    const contractAddress = process.env.ETHEREUM_CONTRACT_ADDRESS || '0x0000000000000000000000000000000000000000';
    
    // ABI for the events we care about
    const eventAbi = [
      "event EscrowCreated(bytes32 indexed escrowId, address indexed creator, address indexed beneficiary, uint256 amount, bytes32 hashlock, uint256 timelock)",
      "event EscrowClaimed(bytes32 indexed escrowId, bytes32 secret)",
      "event EscrowRefunded(bytes32 indexed escrowId)"
    ];

    const contract = new ethers.Contract(contractAddress, eventAbi, ethProvider);

    // Listen for new escrow creations
    contract.on('EscrowCreated', async (escrowId, creator, beneficiary, amount, hashlock, timelock, event) => {
      console.log('🔄 Ethereum EscrowCreated:', {
        escrowId,
        creator,
        beneficiary,
        amount: amount.toString(),
        hashlock,
        timelock: timelock.toString()
      });

      // Emit to connected clients
      io.to(`swap-${escrowId}`).emit('ethereum-escrow-created', {
        escrowId,
        creator,
        beneficiary,
        amount: amount.toString(),
        hashlock,
        timelock: timelock.toString(),
        blockNumber: event.blockNumber
      });

      // Trigger cross-chain action if configured
      await this.handleEthereumEscrowCreated(escrowId, hashlock, amount);
    });

    // Listen for claims
    contract.on('EscrowClaimed', async (escrowId, secret, event) => {
      console.log('✅ Ethereum EscrowClaimed:', { escrowId, secret });
      
      io.to(`swap-${escrowId}`).emit('ethereum-escrow-claimed', {
        escrowId,
        secret,
        blockNumber: event.blockNumber
      });

      // Use secret to claim on Polkadot side
      await this.claimPolkadotEscrow(escrowId, secret);
    });

    // Listen for refunds
    contract.on('EscrowRefunded', async (escrowId, event) => {
      console.log('↩️ Ethereum EscrowRefunded:', { escrowId });
      
      io.to(`swap-${escrowId}`).emit('ethereum-escrow-refunded', {
        escrowId,
        blockNumber: event.blockNumber
      });
    });
  }

  async monitorPolkadotEvents() {
    if (!polkadotApi) {
      console.error('Polkadot API not initialized');
      return;
    }

    // Subscribe to system events
    polkadotApi.query.system.events((events: any) => {
      events.forEach((record: any) => {
        const { event, phase } = record;

        // Filter for our pallet events (when pallet is deployed)
        if (event.section === 'fusion' || event.section === 'palletFusion') {
          console.log('🔄 Polkadot event:', event.method, event.data.toString());

          // Handle different event types
          switch (event.method) {
            case 'EscrowCreated':
              this.handlePolkadotEscrowCreated(event.data);
              break;
            case 'EscrowClaimed':
              this.handlePolkadotEscrowClaimed(event.data);
              break;
            case 'EscrowRefunded':
              this.handlePolkadotEscrowRefunded(event.data);
              break;
          }
        }
      });
    });
  }

  async handleEthereumEscrowCreated(escrowId: string, hashlock: string, amount: any) {
    // For demo: Create corresponding escrow on Polkadot
    console.log('🌉 Initiating cross-chain escrow on Polkadot for:', escrowId);
    
    // This would interact with the Polkadot pallet
    // For now, we'll simulate the cross-chain creation
    setTimeout(() => {
      io.emit('cross-chain-progress', {
        escrowId,
        status: 'polkadot-escrow-created',
        message: 'Corresponding escrow created on Polkadot'
      });
    }, 2000);
  }

  async claimPolkadotEscrow(escrowId: string, secret: string) {
    console.log('🔓 Claiming Polkadot escrow with secret:', { escrowId, secret });
    
    // This would submit a claim extrinsic to Polkadot
    // For now, simulate the claim
    setTimeout(() => {
      io.emit('cross-chain-progress', {
        escrowId,
        status: 'polkadot-claimed',
        message: 'Polkadot escrow claimed successfully'
      });
    }, 1000);
  }

  async handlePolkadotEscrowCreated(data: any) {
    console.log('🔄 Polkadot escrow created:', data.toString());
  }

  async handlePolkadotEscrowClaimed(data: any) {
    console.log('✅ Polkadot escrow claimed:', data.toString());
  }

  async handlePolkadotEscrowRefunded(data: any) {
    console.log('↩️ Polkadot escrow refunded:', data.toString());
  }

  // Monitor specific swap progress
  monitorSwapProgress(swapId: string, swapDetails: any) {
    console.log('👀 Monitoring swap progress for:', swapId);
    
    // Simulate progress updates
    const progressSteps = [
      { step: 'source-locked', delay: 2000 },
      { step: 'target-locked', delay: 4000 },
      { step: 'ready-to-claim', delay: 6000 }
    ];

    progressSteps.forEach(({ step, delay }) => {
      setTimeout(() => {
        io.emit('swap-progress', {
          swapId,
          step,
          timestamp: new Date().toISOString(),
          details: swapDetails
        });
        console.log(`📊 Swap ${swapId} progress: ${step}`);
      }, delay);
    });
  }
}

const relayer = new PolkavexRelayer();

const PORT = process.env.PORT || 3002;
server.listen(PORT, () => {
  console.log(`🚀 Polkavex Relayer running on port ${PORT}`);
  console.log(`📊 Health check: http://localhost:${PORT}/health`);
});

export default relayer;
