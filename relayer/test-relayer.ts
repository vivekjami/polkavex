import axios from 'axios';
import { io } from 'socket.io-client';

const RELAYER_URL = 'http://localhost:3002';

// Test configuration
const testConfig = {
  secretHash: '0x' + '1'.repeat(64), // 32-byte hash
  amount: '1000000000000000000', // 1 ETH in wei
  token: 'ETH',
  beneficiary: '0x742bE2D29234C4D69ad9A1BE5AaFCa7ba10d6f41',
  sourceChain: 'ethereum',
  targetChain: 'polkadot'
};

async function testRelayer() {
  console.log('🧪 Starting Polkavex Relayer Tests\n');

  try {
    // Test 1: Health Check
    console.log('1️⃣ Testing health endpoint...');
    const healthResponse = await axios.get(`${RELAYER_URL}/health`);
    console.log('✅ Health check:', healthResponse.data);
    console.log('');

    // Test 2: AI Route Suggestion
    console.log('2️⃣ Testing AI route suggestion...');
    const routeResponse = await axios.post(`${RELAYER_URL}/api/suggest-route`, {
      token: testConfig.token,
      amount: testConfig.amount,
      userPreference: 'fast execution'
    });
    console.log('✅ AI route suggestion:', routeResponse.data);
    console.log('');

    // Test 3: WebSocket Connection
    console.log('3️⃣ Testing WebSocket connection...');
    const socket = io(RELAYER_URL);
    
    socket.on('connect', () => {
      console.log('✅ WebSocket connected');
      
      // Subscribe to swap updates
      socket.emit('subscribe-swap', testConfig.secretHash);
      console.log('✅ Subscribed to swap updates');
    });

    // Listen for events
    socket.on('swap-initiated', (data: any) => {
      console.log('📨 Received swap-initiated event:', data);
    });

    socket.on('swap-progress', (data: any) => {
      console.log('📊 Received swap-progress event:', data);
    });

    socket.on('swap-claimed', (data: any) => {
      console.log('🔓 Received swap-claimed event:', data);
    });

    // Test 4: Initiate Swap
    console.log('4️⃣ Testing swap initiation...');
    setTimeout(async () => {
      try {
        const swapResponse = await axios.post(`${RELAYER_URL}/api/initiate-swap`, testConfig);
        console.log('✅ Swap initiated:', swapResponse.data);
        
        const swapId = swapResponse.data.swapId;
        
        // Test 5: Check Swap Status
        setTimeout(async () => {
          console.log('5️⃣ Testing swap status check...');
          const statusResponse = await axios.get(`${RELAYER_URL}/api/swap/${swapId}`);
          console.log('✅ Swap status:', statusResponse.data);
          
          // Test 6: Claim Swap
          setTimeout(async () => {
            console.log('6️⃣ Testing swap claim...');
            const claimResponse = await axios.post(`${RELAYER_URL}/api/claim-swap`, {
              swapId,
              secret: '0x' + '2'.repeat(64) // 32-byte secret
            });
            console.log('✅ Swap claim:', claimResponse.data);
            
            // Clean up
            setTimeout(() => {
              socket.disconnect();
              console.log('\n🎉 All tests completed successfully!');
              process.exit(0);
            }, 3000);
          }, 2000);
        }, 2000);
      } catch (error: any) {
        console.error('❌ Swap test failed:', error.response?.data || error.message);
      }
    }, 2000);

  } catch (error: any) {
    console.error('❌ Test failed:', error.response?.data || error.message);
    process.exit(1);
  }
}

// Run tests
testRelayer();
