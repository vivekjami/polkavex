#!/usr/bin/env node

/**
 * Polkavex Day 6 - Comprehensive System Health Check
 * Validates all components for ETHGlobal submission
 */

const http = require('http');
const https = require('https');

class SystemHealthChecker {
    constructor() {
        this.results = {
            relayer: { status: 'unknown', details: {} },
            ui: { status: 'unknown', details: {} },
            ethereum: { status: 'unknown', details: {} },
            overall: { status: 'unknown', score: 0 }
        };
    }

    async checkEndpoint(url, name) {
        return new Promise((resolve) => {
            const protocol = url.startsWith('https') ? https : http;
            const req = protocol.get(url, (res) => {
                let data = '';
                res.on('data', chunk => data += chunk);
                res.on('end', () => {
                    resolve({
                        status: res.statusCode === 200 ? 'healthy' : 'error',
                        statusCode: res.statusCode,
                        response: data.substring(0, 200)
                    });
                });
            });
            
            req.on('error', (err) => {
                resolve({
                    status: 'error',
                    error: err.message
                });
            });
            
            req.setTimeout(5000, () => {
                req.destroy();
                resolve({
                    status: 'timeout',
                    error: 'Request timeout'
                });
            });
        });
    }

    async checkRelayer() {
        console.log('🔍 Checking Relayer Service...');
        
        // Check health endpoint
        const health = await this.checkEndpoint('http://localhost:3002/health', 'relayer-health');
        
        // Check WebSocket (basic connectivity)
        const wsCheck = await this.checkEndpoint('http://localhost:3002', 'relayer-base');
        
        this.results.relayer = {
            status: health.status === 'healthy' ? 'healthy' : 'error',
            details: {
                health,
                websocket: wsCheck,
                timestamp: new Date().toISOString()
            }
        };
        
        console.log(`   Health: ${health.status}`);
        console.log(`   WebSocket: ${wsCheck.status}`);
    }

    async checkUI() {
        console.log('🔍 Checking UI Service...');
        
        const ui = await this.checkEndpoint('http://localhost:3000', 'ui');
        
        this.results.ui = {
            status: ui.status === 'healthy' ? 'healthy' : 'error',
            details: {
                endpoint: ui,
                timestamp: new Date().toISOString()
            }
        };
        
        console.log(`   UI Status: ${ui.status}`);
    }

    async checkEthereum() {
        console.log('🔍 Checking Ethereum Node...');
        
        const hardhat = await this.checkEndpoint('http://127.0.0.1:8545', 'hardhat');
        
        this.results.ethereum = {
            status: hardhat.status === 'healthy' ? 'healthy' : 'error',
            details: {
                hardhat,
                timestamp: new Date().toISOString()
            }
        };
        
        console.log(`   Hardhat Node: ${hardhat.status}`);
    }

    calculateScore() {
        let score = 0;
        let total = 0;
        
        Object.keys(this.results).forEach(key => {
            if (key !== 'overall') {
                total += 100;
                if (this.results[key].status === 'healthy') {
                    score += 100;
                } else if (this.results[key].status === 'warning') {
                    score += 50;
                }
            }
        });
        
        return Math.round((score / total) * 100);
    }

    async runHealthCheck() {
        console.log('🚀 Polkavex System Health Check - Day 6');
        console.log('=====================================\n');
        
        await this.checkRelayer();
        await this.checkUI();
        await this.checkEthereum();
        
        const score = this.calculateScore();
        this.results.overall = {
            status: score >= 80 ? 'healthy' : score >= 60 ? 'warning' : 'critical',
            score,
            timestamp: new Date().toISOString()
        };
        
        console.log('\n📊 Health Check Results:');
        console.log('========================');
        console.log(`Overall Score: ${score}%`);
        console.log(`System Status: ${this.results.overall.status.toUpperCase()}`);
        
        console.log('\nComponent Details:');
        Object.keys(this.results).forEach(key => {
            if (key !== 'overall') {
                console.log(`  ${key}: ${this.results[key].status}`);
            }
        });
        
        if (score >= 80) {
            console.log('\n✅ System is ready for ETHGlobal submission!');
        } else if (score >= 60) {
            console.log('\n⚠️  System has some issues but may be submittable');
        } else {
            console.log('\n❌ System needs attention before submission');
        }
        
        return this.results;
    }
}

// Run the health check
const checker = new SystemHealthChecker();
checker.runHealthCheck().then(results => {
    process.exit(results.overall.score >= 60 ? 0 : 1);
}).catch(err => {
    console.error('Health check failed:', err);
    process.exit(1);
});
