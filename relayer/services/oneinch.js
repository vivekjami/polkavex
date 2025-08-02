/**
 * 1inch API Integration Service
 * Provides price quotes, route optimization, and Fusion+ order management
 */

const axios = require('axios');

class OneInchService {
  constructor(apiKey, chainId = 1) {
    this.apiKey = apiKey;
    this.chainId = chainId;
    this.baseUrl = 'https://api.1inch.dev';
    this.headers = {
      'Authorization': `Bearer ${apiKey}`,
      'accept': 'application/json',
      'content-type': 'application/json'
    };
  }

  /**
   * Get token prices from 1inch
   */
  async getTokenPrices(tokens) {
    try {
      const response = await axios.get(
        `${this.baseUrl}/price/v1.1/${this.chainId}/${tokens.join(',')}`,
        { headers: this.headers }
      );
      return response.data;
    } catch (error) {
      console.error('1inch price fetch error:', error.message);
      return null;
    }
  }

  /**
   * Get optimal swap quote from 1inch
   */
  async getSwapQuote(fromToken, toToken, amount, slippage = 1) {
    try {
      const params = new URLSearchParams({
        src: fromToken,
        dst: toToken,
        amount: amount,
        fee: '0',
        gasPrice: 'fast',
        slippage: slippage.toString(),
        disableEstimate: 'true',
        allowPartialFill: 'true'
      });

      const response = await axios.get(
        `${this.baseUrl}/swap/v6.0/${this.chainId}/quote?${params}`,
        { headers: this.headers }
      );
      
      return {
        success: true,
        data: response.data,
        estimatedGas: response.data.gas,
        minReturn: response.data.dstAmount,
        protocols: response.data.protocols
      };
    } catch (error) {
      console.error('1inch quote error:', error.message);
      return {
        success: false,
        error: error.message,
        fallback: true
      };
    }
  }

  /**
   * Get optimal swap transaction data
   */
  async getSwapTransaction(fromToken, toToken, amount, fromAddress, slippage = 1) {
    try {
      const params = new URLSearchParams({
        src: fromToken,
        dst: toToken,
        amount: amount,
        from: fromAddress,
        slippage: slippage.toString(),
        disableEstimate: 'true',
        allowPartialFill: 'true'
      });

      const response = await axios.get(
        `${this.baseUrl}/swap/v6.0/${this.chainId}/swap?${params}`,
        { headers: this.headers }
      );
      
      return {
        success: true,
        transaction: response.data.tx,
        protocols: response.data.protocols,
        gas: response.data.tx.gas,
        gasPrice: response.data.tx.gasPrice
      };
    } catch (error) {
      console.error('1inch swap transaction error:', error.message);
      return {
        success: false,
        error: error.message
      };
    }
  }

  /**
   * Create Fusion+ order for cross-chain compatibility
   */
  async createFusionOrder(orderData) {
    try {
      const {
        makerAsset,
        takerAsset,
        makerAmount,
        takerAmount,
        maker,
        receiver,
        allowedSender,
        makingAmount,
        takingAmount
      } = orderData;

      // Fusion+ order structure
      const fusionOrder = {
        salt: Date.now().toString(),
        makerAsset,
        takerAsset,
        maker,
        receiver: receiver || maker,
        allowedSender: allowedSender || '0x0000000000000000000000000000000000000000',
        makingAmount: makingAmount || makerAmount,
        takingAmount: takingAmount || takerAmount,
        offsets: '0x',
        interactions: '0x'
      };

      // In a full implementation, this would use the Fusion+ SDK to sign and submit
      // For now, we return the structured order
      return {
        success: true,
        order: fusionOrder,
        orderHash: this.generateOrderHash(fusionOrder),
        message: 'Fusion+ order created (ready for cross-chain execution)'
      };
    } catch (error) {
      console.error('Fusion+ order creation error:', error.message);
      return {
        success: false,
        error: error.message
      };
    }
  }

  /**
   * Get supported tokens for the current chain
   */
  async getSupportedTokens() {
    try {
      const response = await axios.get(
        `${this.baseUrl}/swap/v6.0/${this.chainId}/tokens`,
        { headers: this.headers }
      );
      return response.data.tokens;
    } catch (error) {
      console.error('1inch tokens fetch error:', error.message);
      return {};
    }
  }

  /**
   * Check liquidity for a given token pair
   */
  async checkLiquidity(fromToken, toToken, amount) {
    try {
      const quote = await this.getSwapQuote(fromToken, toToken, amount);
      if (quote.success) {
        const liquidityScore = this.calculateLiquidityScore(quote.data);
        return {
          hasLiquidity: true,
          score: liquidityScore,
          protocols: quote.data.protocols,
          priceImpact: this.calculatePriceImpact(quote.data)
        };
      }
      return { hasLiquidity: false, score: 0 };
    } catch (error) {
      return { hasLiquidity: false, score: 0, error: error.message };
    }
  }

  /**
   * Enhanced route optimization combining 1inch with parachain routing
   */
  async getOptimizedRoute(fromToken, toToken, amount, userPreference = 'best_price') {
    try {
      // Get 1inch quote for comparison
      const inchQuote = await this.getSwapQuote(fromToken, toToken, amount);
      
      // Calculate route score based on user preference
      let routeRecommendation = {
        use1inch: false,
        useCrossChain: true,
        reason: 'Cross-chain routing optimal for parachain benefits'
      };

      if (inchQuote.success) {
        const inchPriceImpact = this.calculatePriceImpact(inchQuote.data);
        const inchGasCost = parseInt(inchQuote.data.gas) * 20; // Estimate gas cost in gwei
        
        // Decision logic based on user preference
        switch (userPreference) {
          case 'lowest_gas':
            routeRecommendation.use1inch = inchGasCost < 50000; // Threshold
            break;
          case 'best_price':
            routeRecommendation.use1inch = inchPriceImpact < 0.1; // Less than 0.1% impact
            break;
          case 'fastest':
            routeRecommendation.use1inch = true; // 1inch typically faster
            break;
          default:
            // Keep cross-chain default
            break;
        }

        return {
          success: true,
          oneInchOption: {
            available: true,
            estimatedOutput: inchQuote.data.dstAmount,
            priceImpact: inchPriceImpact,
            gasCost: inchGasCost,
            protocols: inchQuote.data.protocols
          },
          crossChainOption: {
            available: true,
            parachain: 'Acala', // Would be determined by AI
            estimatedTime: '30-60 seconds',
            benefits: ['Yield opportunities', 'Lower long-term costs']
          },
          recommendation: routeRecommendation
        };
      }

      // Fallback to cross-chain only
      return {
        success: true,
        oneInchOption: { available: false, reason: 'No 1inch liquidity' },
        crossChainOption: {
          available: true,
          parachain: 'Acala',
          estimatedTime: '30-60 seconds',
          benefits: ['Cross-chain composability']
        },
        recommendation: { use1inch: false, useCrossChain: true, reason: '1inch unavailable' }
      };
    } catch (error) {
      console.error('Route optimization error:', error.message);
      return {
        success: false,
        error: error.message,
        recommendation: { use1inch: false, useCrossChain: true, reason: 'Fallback to cross-chain' }
      };
    }
  }

  /**
   * Helper: Calculate liquidity score
   */
  calculateLiquidityScore(quoteData) {
    // Simple scoring based on number of protocols and estimated output
    const protocolCount = quoteData.protocols ? quoteData.protocols.length : 0;
    const hasOutput = quoteData.dstAmount && parseInt(quoteData.dstAmount) > 0;
    return Math.min(100, (protocolCount * 20) + (hasOutput ? 40 : 0));
  }

  /**
   * Helper: Calculate price impact
   */
  calculatePriceImpact(quoteData) {
    // Simplified price impact calculation
    // In reality, this would compare against market rates
    if (!quoteData.dstAmount || !quoteData.srcAmount) return 0;
    
    const inputAmount = parseInt(quoteData.srcAmount);
    const outputAmount = parseInt(quoteData.dstAmount);
    
    // This is a simplified calculation - real implementation would fetch market prices
    return Math.abs((outputAmount / inputAmount - 1) * 100);
  }

  /**
   * Helper: Generate order hash for Fusion+ orders
   */
  generateOrderHash(order) {
    // Simplified hash generation - real implementation would use proper encoding
    const orderString = JSON.stringify(order);
    return `0x${Buffer.from(orderString).toString('hex').slice(0, 64)}`;
  }

  /**
   * Health check for 1inch API
   */
  async healthCheck() {
    try {
      // Use tokens endpoint as health check since healthcheck might not exist
      const response = await axios.get(
        `${this.baseUrl}/token/v1.2/${this.chainId}`,
        { headers: this.headers, timeout: 5000 }
      );
      return {
        status: 'connected',
        chainId: this.chainId,
        apiVersion: 'v6.0',
        tokensAvailable: Object.keys(response.data).length,
        lastCheck: new Date().toISOString()
      };
    } catch (error) {
      return {
        status: 'disconnected',
        error: error.message,
        lastCheck: new Date().toISOString()
      };
    }
  }
}

module.exports = OneInchService;
