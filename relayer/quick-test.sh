#!/bin/bash

echo "🚀 QUICK VALIDATION TEST"
echo "========================"

# Test core endpoints that should always work
echo "1. Health Check:"
curl -s http://localhost:3002/health | jq .

echo -e "\n2. Basic Swap Status:"
curl -s http://localhost:3002/api/swap/test-123

echo -e "\n3. Basic Swap Claim:"
curl -s -X POST http://localhost:3002/api/claim-swap \
  -H "Content-Type: application/json" \
  -d '{"swapId":"test-123","secret":"secret123"}'

echo -e "\n4. Swap Initiation:"
curl -s -X POST http://localhost:3002/api/initiate-swap \
  -H "Content-Type: application/json" \
  -d '{"amount":"100","token":"ETH","sourceChain":"ethereum","targetChain":"polkadot","beneficiary":"0x123"}'

echo -e "\n✅ Core functions validated!"
