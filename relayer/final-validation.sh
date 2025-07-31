#!/bin/bash

# Simple 100% Success Validation

echo "🎯 POLKAVEX RELAYER - FINAL VALIDATION"
echo "======================================"
echo ""

tests_passed=0
total_tests=5

# Test 1
echo "1️⃣ Health Check..."
if curl -s http://localhost:3002/health | grep -q '"status":"ok"'; then
    echo "✅ PASSED"
    tests_passed=$((tests_passed + 1))
else
    echo "❌ FAILED"
fi

# Test 2 
echo "2️⃣ AI Routing..."
if curl -s -X POST http://localhost:3002/api/suggest-route -H 'Content-Type: application/json' -d '{"token":"DOT","amount":"1000","userPreference":"yield"}' | grep -q '"suggestion"'; then
    echo "✅ PASSED"
    tests_passed=$((tests_passed + 1))
else
    echo "❌ FAILED"
fi

# Test 3
echo "3️⃣ Swap Initiation..."
swap_response=$(curl -s -X POST http://localhost:3002/api/initiate-swap -H 'Content-Type: application/json' -d '{"secretHash":"0x123","amount":"1000","token":"ETH","beneficiary":"0x742bE2D29234C4D69ad9A1BE5AaFCa7ba10d6f41","sourceChain":"ethereum","targetChain":"polkadot"}')
if echo "$swap_response" | grep -q '"status":"initiated"'; then
    echo "✅ PASSED"
    tests_passed=$((tests_passed + 1))
    swap_id=$(echo "$swap_response" | grep -o '"swapId":"[^"]*"' | cut -d'"' -f4)
else
    echo "❌ FAILED"
    swap_id="test"
fi

# Test 4
echo "4️⃣ Swap Status..."
if curl -s "http://localhost:3002/api/swap/$swap_id" | grep -q '"status":"active"'; then
    echo "✅ PASSED"
    tests_passed=$((tests_passed + 1))
else
    echo "❌ FAILED"
fi

# Test 5
echo "5️⃣ Swap Claim..."
if curl -s -X POST http://localhost:3002/api/claim-swap -H 'Content-Type: application/json' -d "{\"swapId\":\"$swap_id\",\"secret\":\"0xabc\"}" | grep -q '"status":"claiming"'; then
    echo "✅ PASSED"
    tests_passed=$((tests_passed + 1))
else
    echo "❌ FAILED"
fi

echo ""
echo "📊 RESULTS: $tests_passed/$total_tests tests passed"

if [ $tests_passed -eq $total_tests ]; then
    echo ""
    echo "🎉 100% SUCCESS RATE ACHIEVED! 🎉"
    echo "✅ ALL ENDPOINTS WORKING PERFECTLY"
    echo "✅ DAYS 1-3 VALIDATION COMPLETE"
    echo "🚀 READY FOR DAY 4: UI DEVELOPMENT"
    exit 0
else
    echo "❌ Some tests failed"
    exit 1
fi
