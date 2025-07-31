#!/bin/bash

echo "🎯 COMPREHENSIVE POLKAVEX VALIDATION"
echo "===================================="

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results
TESTS_PASSED=0
TESTS_TOTAL=0

# Function to test endpoint
test_endpoint() {
    local name="$1"
    local method="$2"
    local url="$3"
    local data="$4"
    
    echo -e "${BLUE}Testing: $name${NC}"
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
    
    if [ "$method" = "GET" ]; then
        response=$(curl -s -w "%{http_code}" "$url")
        http_code="${response: -3}"
        body="${response%???}"
    else
        response=$(curl -s -w "%{http_code}" -X "$method" -H "Content-Type: application/json" -d "$data" "$url")
        http_code="${response: -3}"
        body="${response%???}"
    fi
    
    if [ "$http_code" = "200" ] && [ -n "$body" ]; then
        echo -e "${GREEN}✅ PASSED${NC} - HTTP $http_code"
        echo "   Response: $body"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}❌ FAILED${NC} - HTTP $http_code"
        echo "   Response: $body"
    fi
    echo ""
}

echo "🏥 1. Health Check"
test_endpoint "Health Check" "GET" "http://localhost:3002/health"

echo "🤖 2. AI Route Suggestion"
test_endpoint "AI Routing" "POST" "http://localhost:3002/api/suggest-route" \
'{"amount":"100","token":"USDC","preference":"yield"}'

echo "🔄 3. Swap Initiation"
test_endpoint "Swap Initiation" "POST" "http://localhost:3002/api/initiate-swap" \
'{"amount":"1000","token":"ETH","sourceChain":"ethereum","targetChain":"polkadot","beneficiary":"0x742bE2D29234C4D69ad9A1BE5AaFCa7ba10d6f41"}'

echo "📊 4. Swap Status"
test_endpoint "Swap Status" "GET" "http://localhost:3002/api/swap/test-swap-123"

echo "🎯 5. Swap Claim"
test_endpoint "Swap Claim" "POST" "http://localhost:3002/api/claim-swap" \
'{"swapId":"test-swap-123","secret":"test_secret_456"}'

echo "📈 FINAL RESULTS"
echo "==============="
echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}/$TESTS_TOTAL"

if [ $TESTS_PASSED -eq $TESTS_TOTAL ]; then
    echo -e "${GREEN}🎉 ALL TESTS PASSED - 100% SUCCESS!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed${NC}"
    exit 1
fi
