#!/bin/bash

echo "🎯 FINAL RELAYER VERIFICATION"
echo "============================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

passed=0
total=0

test_endpoint() {
    local name="$1"
    local test_cmd="$2"
    local expected="$3"
    
    echo -e "${BLUE}Testing: $name${NC}"
    total=$((total + 1))
    
    result=$(eval "$test_cmd" 2>/dev/null)
    
    if echo "$result" | grep -q "$expected"; then
        echo -e "${GREEN}✅ PASSED${NC}"
        passed=$((passed + 1))
    else
        echo -e "${RED}❌ FAILED${NC}"
        echo "   Expected: $expected"
        echo "   Got: $result"
    fi
    echo ""
}

echo "🔍 Testing Relayer Endpoints..."
echo ""

# Test 1: Health Check
test_endpoint "Health Check" \
    "curl -s http://localhost:3002/health" \
    '"status":"ok"'

# Test 2: Swap Initiation  
test_endpoint "Swap Initiation" \
    "curl -s -X POST http://localhost:3002/api/initiate-swap -H 'Content-Type: application/json' -d '{\"secretHash\":\"0x123abc\",\"amount\":\"100\",\"beneficiary\":\"0x742d35Cc6642C66532C42bB8C0b20F2a3B4b100d\",\"sourceChain\":\"ethereum\",\"targetChain\":\"polkadot\"}'" \
    '"status":"initiated"'

# Test 3: Swap Status
test_endpoint "Swap Status Check" \
    "curl -s 'http://localhost:3002/api/swap/test-id-123'" \
    '"status":"active"'

# Test 4: Claim Swap
test_endpoint "Swap Claim" \
    "curl -s -X POST http://localhost:3002/api/claim-swap -H 'Content-Type: application/json' -d '{\"swapId\":\"test123\",\"secret\":\"testsecret\"}'" \
    '"status":"claiming"'

echo "📊 FINAL RESULTS"
echo "================"
echo -e "Passed: ${GREEN}$passed${NC}/$total tests"

if [ $passed -eq $total ]; then
    echo -e "${GREEN}🎉 ALL TESTS PASSED - RELAYER 100% VERIFIED!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed${NC}"
    exit 1
fi
