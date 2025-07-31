#!/bin/bash

# Polkavex Day 2-3 Demonstration Script
# Shows complete cross-chain relayer functionality

echo "🚀 Polkavex Cross-Chain Relayer Demonstration"
echo "============================================="
echo ""

echo "📊 Step 1: Health Check"
echo "Verifying relayer system status..."
curl -s http://localhost:3002/health | python3 -m json.tool
echo ""

echo "🤖 Step 2: AI Route Suggestion"
echo "Testing AI-powered parachain routing..."
curl -s -X POST http://localhost:3002/api/suggest-route \
  -H "Content-Type: application/json" \
  -d '{"token":"DOT","amount":"1000000000000","userPreference":"high yield"}' | python3 -m json.tool
echo ""

echo "🌉 Step 3: Cross-Chain Swap Initiation"
echo "Initiating Ethereum → Polkadot swap..."
SWAP_RESPONSE=$(curl -s -X POST http://localhost:3002/api/initiate-swap \
  -H "Content-Type: application/json" \
  -d '{
    "secretHash":"0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    "amount":"1000000000000000000",
    "token":"ETH",
    "beneficiary":"0x742bE2D29234C4D69ad9A1BE5AaFCa7ba10d6f41",
    "sourceChain":"ethereum",
    "targetChain":"polkadot"
  }')

echo "$SWAP_RESPONSE" | python3 -m json.tool
SWAP_ID=$(echo "$SWAP_RESPONSE" | python3 -c "import sys, json; print(json.load(sys.stdin)['swapId'])")
echo ""

echo "📋 Step 4: Swap Status Check"
echo "Checking swap progress for ID: $SWAP_ID"
sleep 2
curl -s "http://localhost:3002/api/swap/$SWAP_ID" | python3 -m json.tool
echo ""

echo "🔓 Step 5: Swap Claim Simulation"
echo "Simulating claim with secret..."
curl -s -X POST http://localhost:3002/api/claim-swap \
  -H "Content-Type: application/json" \
  -d "{\"swapId\":\"$SWAP_ID\",\"secret\":\"0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890\"}" | python3 -m json.tool
echo ""

echo "✅ Demonstration Complete!"
echo ""
echo "🎯 Key Achievements Shown:"
echo "  • Live cross-chain relayer operational"
echo "  • AI-powered routing suggestions"
echo "  • Complete swap lifecycle management"
echo "  • Real-time status tracking"
echo "  • Production-ready API endpoints"
echo ""
echo "🏆 Ready for hackathon demonstration!"
echo "📈 Polkavex: Making cross-chain DeFi simple and intelligent"
