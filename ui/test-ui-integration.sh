#!/bin/bash

# Polkavex UI Integration Test Script
# Tests the full UI → Relayer → Backend integration

echo "🚀 Starting Polkavex UI Integration Tests"
echo "========================================="

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TESTS_RUN=0
TESTS_PASSED=0

run_test() {
    TESTS_RUN=$((TESTS_RUN + 1))
    echo -e "\n🧪 Test $TESTS_RUN: $1"
    echo "-------------------"
}

pass_test() {
    TESTS_PASSED=$((TESTS_PASSED + 1))
    echo -e "${GREEN}✅ PASSED${NC}"
}

fail_test() {
    echo -e "${RED}❌ FAILED: $1${NC}"
}

warn_test() {
    echo -e "${YELLOW}⚠️  WARNING: $1${NC}"
}

# Test 1: Check relayer health
run_test "Relayer Health Check"
HEALTH_RESPONSE=$(curl -s http://localhost:3002/health)
if echo "$HEALTH_RESPONSE" | grep -q '"status":"ok"'; then
    echo "Relayer is healthy: $HEALTH_RESPONSE"
    pass_test
else
    fail_test "Relayer not responding or unhealthy"
    exit 1
fi

# Test 2: Check UI server
run_test "UI Server Check"
UI_RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3003)
if [ "$UI_RESPONSE" = "200" ]; then
    echo "UI server is running on port 3003"
    pass_test
else
    fail_test "UI server not responding"
    exit 1
fi

# Test 3: Test AI routing endpoint
run_test "AI Route Suggestion"
ROUTE_RESPONSE=$(curl -s -X POST http://localhost:3002/api/suggest-route \
  -H "Content-Type: application/json" \
  -d '{
    "amount": "1000",
    "token": "USDC",
    "sourceChain": "ethereum", 
    "targetChain": "polkadot"
  }')

if echo "$ROUTE_RESPONSE" | grep -q '"suggestion"'; then
    echo "AI routing working: $(echo "$ROUTE_RESPONSE" | jq -r '.suggestion.reason')"
    pass_test
else
    fail_test "AI routing failed"
fi

# Test 4: Test swap initiation
run_test "Swap Initiation"
SECRET_HASH="0x$(echo -n "test_secret_$(date +%s)" | sha256sum | cut -d' ' -f1)"
TIMELOCK=$(($(date +%s) + 3600))

SWAP_RESPONSE=$(curl -s -X POST http://localhost:3002/api/initiate-swap \
  -H "Content-Type: application/json" \
  -d "{
    \"amount\": \"500\",
    \"token\": \"USDC\",
    \"sourceChain\": \"ethereum\",
    \"targetChain\": \"polkadot\",
    \"beneficiary\": \"5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY\",
    \"secretHash\": \"$SECRET_HASH\",
    \"timelock\": $TIMELOCK
  }")

if echo "$SWAP_RESPONSE" | grep -q '"status":"initiated"'; then
    SWAP_ID=$(echo "$SWAP_RESPONSE" | jq -r '.swapDetails.swapId')
    echo "Swap initiated successfully: $SWAP_ID"
    pass_test
    
    # Test 5: Check swap status
    run_test "Swap Status Check"
    sleep 2
    STATUS_RESPONSE=$(curl -s http://localhost:3002/api/swap/$SWAP_ID)
    
    if echo "$STATUS_RESPONSE" | grep -q '"swapId"'; then
        echo "Swap status retrieved: $(echo "$STATUS_RESPONSE" | jq -r '.status')"
        pass_test
    else
        fail_test "Could not retrieve swap status"
    fi
else
    fail_test "Swap initiation failed"
fi

# Test 6: WebSocket connection test
run_test "WebSocket Connection Test"
if command -v node >/dev/null 2>&1; then
    # Create a simple WebSocket test
    cat > /tmp/websocket_test.js << 'EOF'
const io = require('socket.io-client');
const socket = io('http://localhost:3002');

socket.on('connect', () => {
    console.log('✅ WebSocket connected successfully');
    socket.disconnect();
    process.exit(0);
});

socket.on('connect_error', (error) => {
    console.log('❌ WebSocket connection failed:', error.message);
    process.exit(1);
});

setTimeout(() => {
    console.log('❌ WebSocket connection timeout');
    process.exit(1);
}, 5000);
EOF

    cd /home/vivek/vivek/polkavex/relayer
    if timeout 10s node /tmp/websocket_test.js 2>/dev/null; then
        pass_test
    else
        fail_test "WebSocket connection failed"
    fi
    rm -f /tmp/websocket_test.js
else
    warn_test "Node.js not available for WebSocket test"
fi

# Test 7: UI Components Test (Static Analysis)
run_test "UI Components Analysis"
if grep -q "Chart" /home/vivek/vivek/polkavex/ui/src/App.tsx; then
    echo "Charts integration found in UI"
    if grep -q "socket.io-client" /home/vivek/vivek/polkavex/ui/src/App.tsx; then
        echo "WebSocket integration found in UI"
        if grep -q "progress-container" /home/vivek/vivek/polkavex/ui/src/App.css; then
            echo "Progress indicator styling found"
            pass_test
        else
            fail_test "Progress indicator missing"
        fi
    else
        fail_test "WebSocket integration missing"
    fi
else
    fail_test "Charts integration missing"
fi

# Summary
echo ""
echo "========================================="
echo "🏁 Test Summary"
echo "========================================="
echo "Tests Run: $TESTS_RUN"
echo "Tests Passed: $TESTS_PASSED"
echo "Tests Failed: $((TESTS_RUN - TESTS_PASSED))"

if [ $TESTS_PASSED -eq $TESTS_RUN ]; then
    echo -e "${GREEN}🎉 ALL TESTS PASSED! UI Integration is working perfectly.${NC}"
    echo ""
    echo "✅ Relayer is healthy and responsive"
    echo "✅ UI is running and accessible"
    echo "✅ AI routing is functional"
    echo "✅ Swap lifecycle is working"
    echo "✅ WebSocket real-time updates are ready"
    echo "✅ UI components are properly integrated"
    echo ""
    echo "🌐 Access your Polkavex UI at: http://localhost:3003"
    echo "🔧 Relayer API available at: http://localhost:3002"
    exit 0
else
    echo -e "${RED}❌ SOME TESTS FAILED. Please check the issues above.${NC}"
    exit 1
fi
