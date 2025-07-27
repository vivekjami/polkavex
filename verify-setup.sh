#!/bin/bash

# Polkavex Day 0 Setup Verification Script
# Run this script to verify all components are working

echo "🚀 Polkavex Day 0 Setup Verification"
echo "===================================="

# Check required tools
echo "📋 Checking required tools..."

if command -v node &> /dev/null; then
    echo "✅ Node.js: $(node --version)"
else
    echo "❌ Node.js not found"
    exit 1
fi

if command -v npm &> /dev/null; then
    echo "✅ npm: $(npm --version)"
else
    echo "❌ npm not found"
    exit 1
fi

if command -v cargo &> /dev/null; then
    echo "✅ Rust/Cargo: $(cargo --version)"
else
    echo "❌ Rust/Cargo not found"
    exit 1
fi

# Check project structure
echo ""
echo "📁 Checking project structure..."

dirs=("ethereum" "polkadot" "ui" "relayer")
for dir in "${dirs[@]}"; do
    if [ -d "$dir" ]; then
        echo "✅ $dir/ directory exists"
    else
        echo "❌ $dir/ directory missing"
        exit 1
    fi
done

# Test Ethereum contracts
echo ""
echo "🔗 Testing Ethereum contracts..."
cd ethereum
if npx hardhat compile &> /dev/null; then
    echo "✅ Ethereum contracts compile successfully"
else
    echo "❌ Ethereum contract compilation failed"
    cd ..
    exit 1
fi
cd ..

# Test Polkadot pallet
echo ""
echo "⚫ Testing Polkadot pallet..."
cd polkadot
if cargo check &> /dev/null; then
    echo "✅ Polkadot pallet compiles successfully"
else
    echo "❌ Polkadot pallet compilation failed"
    cd ..
    exit 1
fi
cd ..

# Test React UI
echo ""
echo "⚛️  Testing React UI..."
cd ui
if npm run build &> /dev/null; then
    echo "✅ React UI builds successfully"
else
    echo "❌ React UI build failed"
    cd ..
    exit 1
fi
cd ..

# Test Relayer
echo ""
echo "🔄 Testing Relayer..."
cd relayer
if node -e "require('./index.js')" &> /dev/null; then
    echo "✅ Relayer loads successfully"
else
    echo "❌ Relayer has errors"
    cd ..
    exit 1
fi
cd ..

echo ""
echo "🎉 Day 0 Setup Complete!"
echo "======================="
echo ""
echo "✅ All components verified and working"
echo "📅 Today: July 27, 2025"
echo "⏰ Next: Start Day 1 (Ethereum Fusion+ Contracts)"
echo ""
echo "🔧 Quick Start Commands:"
echo "  npm run install:all    # Install all dependencies"
echo "  npm run dev           # Start development servers"
echo "  npm run test          # Run all tests"
echo ""
echo "📚 Day 1 Focus Areas:"
echo "  - Complete Ethereum escrow contract with Fusion+ integration"
echo "  - Add 1inch SDK integration for order signing"
echo "  - Deploy and test on Sepolia testnet"
echo "  - Create comprehensive test suite"
echo ""
echo "🎯 Remember: You have until August 2 to complete the project!"
echo "💪 Stay focused and build something amazing!"
