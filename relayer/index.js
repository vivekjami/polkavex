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

module.exports = app;
