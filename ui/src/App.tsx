import React, { useState, useEffect } from 'react';
import { ethers } from 'ethers';
import io from 'socket.io-client';
import { Chart as ChartJS, CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip, Legend, ArcElement } from 'chart.js';
import { Line, Doughnut } from 'react-chartjs-2';
import './App.css';

// MetaMask type declarations
declare global {
  interface Window {
    ethereum?: any;
  }
}

// Register Chart.js components
ChartJS.register(CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip, Legend, ArcElement);

// Initialize socket connection to relayer
const socket = io('http://localhost:3002');

interface SwapData {
  swapId: string;
  status: string;
  amount: string;
  token: string;
  targetToken?: string;
  sourceChain: string;
  targetChain: string;
  createdAt: string;
  progress?: any[];
  txHash?: string;
  estimatedTime?: string;
  estimatedGas?: string;
  route?: {
    parachain: string;
    reason: string;
  };
}

interface AnalyticsData {
  totalSwaps: number;
  totalVolume: string;
  successRate: number;
  avgTime: string;
  chartData: any;
  recentSwaps: SwapData[];
}

// Available tokens for swapping with asset type classification
const TOKENS = [
  { symbol: 'ETH', name: 'Ethereum', icon: '⚡', type: 'native', address: '0x0' },
  { symbol: 'USDC', name: 'USD Coin', icon: '💲', type: 'stablecoin', address: '0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238' },
  { symbol: 'USDT', name: 'Tether USD', icon: '💰', type: 'stablecoin', address: '0x7169D38820dfd117C3FA1f22a697dBA58d90BA06' },
  { symbol: 'DAI', name: 'Dai Stablecoin', icon: '🪙', type: 'stablecoin', address: '0xFF34B3d4Aee8ddCd6F9AFFFB6Fe49bD371b8a357' },
  { symbol: 'DOT', name: 'Polkadot', icon: '🔴', type: 'native', address: '0x0' },
  { symbol: 'ACA', name: 'Acala', icon: '🌊', type: 'token', address: '0x0' },
  { symbol: 'NFT', name: 'NFT Collection', icon: '🎨', type: 'nft', address: '0x0' }
];

// Asset type definitions for enhanced UI
const ASSET_TYPES = [
  { id: 'all', name: 'All Assets', icon: '🌐' },
  { id: 'native', name: 'Native Tokens', icon: '⚡' },
  { id: 'stablecoin', name: 'Stablecoins', icon: '💲' },
  { id: 'token', name: 'ERC-20 Tokens', icon: '🪙' },
  { id: 'nft', name: 'NFTs', icon: '🎨' }
];

// Available chains
const CHAINS = [
  { id: 'ethereum', name: 'Ethereum', icon: '⚡', color: '#627eea' },
  { id: 'polkadot', name: 'Polkadot', icon: '🔴', color: '#e6007a' },
  { id: 'acala', name: 'Acala', icon: '🌊', color: '#ff4081' },
  { id: 'moonbeam', name: 'Moonbeam', icon: '🌙', color: '#53cbc8' }
];

function App() {
  const [activeTab, setActiveTab] = useState<'swap' | 'dashboard'>('swap');
  const [swapData, setSwapData] = useState<SwapData | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [loading, setLoading] = useState(false);
  const [walletConnected, setWalletConnected] = useState(false);
  const [walletAddress, setWalletAddress] = useState('');
  const [walletBalance, setWalletBalance] = useState('0.0');
  
  // Swap form state
  const [fromChain, setFromChain] = useState('ethereum');
  const [toChain, setToChain] = useState('polkadot');
  const [fromToken, setFromToken] = useState('ETH');
  const [toToken, setToToken] = useState('DOT');
  const [amount, setAmount] = useState('');
  const [recipientAddress, setRecipientAddress] = useState('');
  const [slippage, setSlippage] = useState('0.5');
  const [deadline, setDeadline] = useState('20');
  
  const [analytics, setAnalytics] = useState<AnalyticsData>({
    totalSwaps: 50,
    totalVolume: '248,750 USDC',
    successRate: 100.0,
    avgTime: '4.88s',
    chartData: {
      labels: ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'],
      datasets: [
        {
          label: 'Daily Volume (USDC)',
          data: [15000, 22000, 18000, 32000, 28000, 45000, 52000],
          borderColor: '#e6007a',
          backgroundColor: 'rgba(230, 0, 122, 0.1)',
          tension: 0.4,
          fill: true
        }
      ]
    },
    recentSwaps: [
      {
        swapId: 'swap_abc123',
        status: 'completed',
        amount: '1000',
        token: 'USDC',
        sourceChain: 'ethereum',
        targetChain: 'acala',
        createdAt: new Date(Date.now() - 300000).toISOString(),
        txHash: '0x1234...abcd'
      },
      {
        swapId: 'swap_def456',
        status: 'completed',
        amount: '500',
        token: 'ETH',
        sourceChain: 'polkadot',
        targetChain: 'ethereum',
        createdAt: new Date(Date.now() - 600000).toISOString()
      }
    ]
  });

  // Connect to MetaMask wallet
  const connectWallet = async () => {
    if (typeof window.ethereum !== 'undefined') {
      try {
        setLoading(true);
        const accounts = await window.ethereum.request({ method: 'eth_requestAccounts' });
        const provider = new ethers.BrowserProvider(window.ethereum);
        const balance = await provider.getBalance(accounts[0]);
        
        setWalletConnected(true);
        setWalletAddress(accounts[0]);
        setWalletBalance(ethers.formatEther(balance));
        
        // Auto-fill recipient if empty
        if (!recipientAddress) {
          setRecipientAddress(accounts[0]);
        }
      } catch (error) {
        console.error('Failed to connect wallet:', error);
        alert('Failed to connect wallet. Please try again.');
      } finally {
        setLoading(false);
      }
    } else {
      alert('Please install MetaMask to connect your wallet!');
    }
  };

  // Disconnect wallet
  const disconnectWallet = () => {
    setWalletConnected(false);
    setWalletAddress('');
    setWalletBalance('0.0');
  };

  // Swap tokens in form
  const swapTokens = () => {
    const tempFromChain = fromChain;
    const tempFromToken = fromToken;
    setFromChain(toChain);
    setFromToken(toToken);
    setToChain(tempFromChain);
    setToToken(tempFromToken);
  };

  // Enhanced form validation
  const validateForm = () => {
    if (!amount || parseFloat(amount) <= 0) {
      alert('Please enter a valid amount');
      return false;
    }
    if (!recipientAddress) {
      alert('Please enter a recipient address');
      return false;
    }
    if (fromChain === toChain) {
      alert('Source and destination chains must be different');
      return false;
    }
    return true;
  };

  const initiateSwap = async () => {
    if (!validateForm()) {
      return;
    }

    setLoading(true);
    try {
      const secretHash = ethers.keccak256(ethers.toUtf8Bytes(`secret_${Date.now()}_${Math.random()}`));
      
      const swapRequest = {
        amount,
        fromToken,
        toToken,
        fromChain,
        toChain,
        beneficiary: recipientAddress,
        assetType: getAssetType(fromToken),
        secretHash,
        timelock: Math.floor(Date.now() / 1000) + parseInt(deadline) * 60,
        slippage: parseFloat(slippage)
      };
      
      const response = await fetch('http://localhost:3002/api/initiate-swap', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(swapRequest)
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const result = await response.json();
      console.log('Swap initiated:', result);
      setSwapData(result.swapDetails || result);
      setActiveTab('dashboard'); // Switch to dashboard to show progress
      
      // Fetch updated analytics
      await fetchAnalytics();
      
    } catch (error) {
      console.error('Swap initiation failed:', error);
      alert('Failed to initiate swap. Please check your connection and try again.');
    } finally {
      setLoading(false);
    }
  };

  // Determine asset type for enhanced routing
  const getAssetType = (token: string) => {
    if (['USDC', 'USDT', 'DAI'].includes(token)) return 'stablecoin';
    if (token.includes('NFT')) return 'nft';
    return 'native';
  };

  const fetchAnalytics = async () => {
    try {
      await fetch('http://localhost:3002/health');
      // Analytics are already set in state - in production this would fetch real data
    } catch (error) {
      console.log('Relayer connection status: checking...');
    }
  };

  // Check connection status
  useEffect(() => {
    const checkConnection = async () => {
      try {
        const response = await fetch('http://localhost:3002/health');
        setIsConnected(response.ok);
      } catch {
        setIsConnected(false);
      }
    };

    checkConnection();
    const interval = setInterval(checkConnection, 10000);
    
    // Socket event listeners
    socket.on('connect', () => setIsConnected(true));
    socket.on('disconnect', () => setIsConnected(false));
    socket.on('swap-update', (data) => {
      if (swapData && data.swapId === swapData.swapId) {
        setSwapData({ ...swapData, ...data });
      }
    });

    return () => {
      clearInterval(interval);
      socket.off('connect');
      socket.off('disconnect');
      socket.off('swap-update');
    };
  }, [swapData]);

  // Fetch analytics on tab change
  useEffect(() => {
    if (activeTab === 'dashboard') {
      fetchAnalytics();
    }
  }, [activeTab]);

  const renderSwapInterface = () => (
    <div className="swap-container">
      {/* Wallet Connection Section */}
      <div className="wallet-section">
        {!walletConnected ? (
          <button onClick={connectWallet} className="wallet-connect-btn" disabled={loading}>
            {loading ? '🔄 Connecting...' : '🦊 Connect MetaMask'}
          </button>
        ) : (
          <div className="wallet-info">
            <div className="wallet-address">
              <span className="wallet-icon">✅</span>
              <span>Connected: {walletAddress.slice(0, 6)}...{walletAddress.slice(-4)}</span>
              <span className="balance">{parseFloat(walletBalance).toFixed(4)} ETH</span>
              <button onClick={disconnectWallet} className="disconnect-btn">Disconnect</button>
            </div>
          </div>
        )}
      </div>

      <div className="swap-header">
        <h2>🌉 Cross-Chain Swap</h2>
        <p>Bridge assets between Ethereum and Polkadot with AI-optimized routing</p>
      </div>

      <div className="swap-form">
        {/* From Section */}
        <div className="swap-section">
          <label className="section-label">From</label>
          <div className="token-input">
            <div className="token-selector">
              <select
                value={fromChain}
                onChange={(e) => setFromChain(e.target.value)}
                className="chain-select"
              >
                {CHAINS.map(chain => (
                  <option key={chain.id} value={chain.id}>
                    {chain.icon} {chain.name}
                  </option>
                ))}
              </select>
              <select
                value={fromToken}
                onChange={(e) => setFromToken(e.target.value)}
                className="token-select"
              >
                {TOKENS.map(token => (
                  <option key={token.symbol} value={token.symbol}>
                    {token.icon} {token.symbol}
                  </option>
                ))}
              </select>
            </div>
            <input
              type="number"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              placeholder="0.00"
              className="amount-input"
              step="0.000001"
              min="0"
            />
          </div>
          {walletConnected && (
            <div className="balance-display">
              Balance: {walletBalance} ETH
            </div>
          )}
        </div>

        {/* Swap Button */}
        <div className="swap-button-container">
          <button onClick={swapTokens} className="swap-direction-btn" title="Swap direction">
            🔄
          </button>
        </div>

        {/* To Section */}
        <div className="swap-section">
          <label className="section-label">To</label>
          <div className="token-input">
            <div className="token-selector">
              <select
                value={toChain}
                onChange={(e) => setToChain(e.target.value)}
                className="chain-select"
              >
                {CHAINS.map(chain => (
                  <option key={chain.id} value={chain.id}>
                    {chain.icon} {chain.name}
                  </option>
                ))}
              </select>
              <select
                value={toToken}
                onChange={(e) => setToToken(e.target.value)}
                className="token-select"
              >
                {TOKENS.map(token => (
                  <option key={token.symbol} value={token.symbol}>
                    {token.icon} {token.symbol}
                  </option>
                ))}
              </select>
            </div>
            <div className="estimated-output">
              ~{amount || '0.00'} {toToken}
            </div>
          </div>
        </div>

        {/* Recipient Address */}
        <div className="input-group full-width">
          <label>Recipient Address</label>
          <input
            type="text"
            value={recipientAddress}
            onChange={(e) => setRecipientAddress(e.target.value)}
            placeholder="0x... or wallet address on destination chain"
            className="address-input"
          />
          {walletConnected && recipientAddress !== walletAddress && (
            <button 
              onClick={() => setRecipientAddress(walletAddress)}
              className="use-wallet-btn"
            >
              Use Connected Wallet
            </button>
          )}
        </div>

        {/* Advanced Settings */}
        <details className="advanced-settings">
          <summary>⚙️ Advanced Settings</summary>
          <div className="advanced-content">
            <div className="setting-row">
              <label>Slippage Tolerance</label>
              <select value={slippage} onChange={(e) => setSlippage(e.target.value)}>
                <option value="0.1">0.1%</option>
                <option value="0.5">0.5%</option>
                <option value="1.0">1.0%</option>
                <option value="3.0">3.0%</option>
              </select>
            </div>
            <div className="setting-row">
              <label>Deadline (minutes)</label>
              <input
                type="number"
                value={deadline}
                onChange={(e) => setDeadline(e.target.value)}
                min="1"
                max="60"
              />
            </div>
          </div>
        </details>

        {/* Swap Button */}
        <button
          onClick={initiateSwap}
          disabled={loading || !amount || !recipientAddress}
          className="swap-execute-btn"
        >
          {loading ? (
            <>🔄 Processing...</>
          ) : (
            <>🚀 Initiate Cross-Chain Swap</>
          )}
        </button>

        {/* Asset Type Info */}
        <div className="asset-info">
          <div className="info-item">
            💡 <strong>AI Routing:</strong> System will automatically select the optimal parachain for your swap
          </div>
          <div className="info-item">
            ⚡ <strong>Asset Type:</strong> {getAssetType(fromToken)} detected - optimized routing enabled
          </div>
          <div className="info-item">
            🔒 <strong>Security:</strong> Hashlock/timelock protection with {deadline}min deadline
          </div>
        </div>
      </div>

      {/* Swap Status Display */}
      {swapData && (
        <div className="swap-status">
          <h3>🔄 Swap Status</h3>
          <div className="status-details">
            <p><span className="label">Swap ID:</span> {swapData.swapId}</p>
            <p><span className="label">Status:</span> 
              <span className={`status-badge ${swapData.status}`}>
                {swapData.status}
              </span>
            </p>
            <p><span className="label">Amount:</span> {swapData.amount} {swapData.token}</p>
            <p><span className="label">Route:</span> {swapData.sourceChain} → {swapData.targetChain}</p>
            {swapData.route && (
              <p><span className="label">AI Route:</span> {swapData.route.parachain} - {swapData.route.reason}</p>
            )}
          </div>
        </div>
      )}
    </div>
  );

  const renderDashboard = () => (
    <div className="dashboard-container">
      <div className="dashboard-header">
        <h2>📊 Analytics Dashboard</h2>
        <p>Real-time insights into cross-chain swap performance</p>
      </div>

      <div className="metrics-grid">
        <div className="metric-card">
          <div className="metric-icon">🔄</div>
          <div className="metric-content">
            <h3>{analytics.totalSwaps}</h3>
            <p>Total Swaps</p>
          </div>
        </div>
        
        <div className="metric-card">
          <div className="metric-icon">💰</div>
          <div className="metric-content">
            <h3>{analytics.totalVolume}</h3>
            <p>Total Volume</p>
          </div>
        </div>
        
        <div className="metric-card">
          <div className="metric-icon">✅</div>
          <div className="metric-content">
            <h3>{analytics.successRate}%</h3>
            <p>Success Rate</p>
          </div>
        </div>
        
        <div className="metric-card">
          <div className="metric-icon">⚡</div>
          <div className="metric-content">
            <h3>{analytics.avgTime}</h3>
            <p>Avg Time</p>
          </div>
        </div>
      </div>

      <div className="charts-section">
        <div className="chart-container">
          <h3>📈 Daily Volume</h3>
          <Line data={analytics.chartData} options={{
            responsive: true,
            plugins: {
              legend: {
                position: 'top' as const,
              },
              title: {
                display: true,
                text: 'Cross-Chain Swap Volume'
              }
            }
          }} />
        </div>

        <div className="chart-container">
          <h3>🎯 Success Rate</h3>
          <Doughnut data={{
            labels: ['Successful', 'Failed'],
            datasets: [{
              data: [analytics.successRate, 100 - analytics.successRate],
              backgroundColor: ['#22c55e', '#ef4444'],
              borderWidth: 0
            }]
          }} options={{
            responsive: true,
            plugins: {
              legend: {
                position: 'bottom' as const
              }
            }
          }} />
        </div>
      </div>

      <div className="recent-swaps">
        <h3>🕒 Recent Swaps</h3>
        <div className="swaps-list">
          {analytics.recentSwaps.map((swap, index) => (
            <div key={index} className="swap-item">
              <div className="swap-info">
                <span className="swap-amount">{swap.amount} {swap.token}</span>
                <span className="swap-route">{swap.sourceChain} → {swap.targetChain}</span>
              </div>
              <div className="swap-meta">
                <span className={`status-badge ${swap.status}`}>{swap.status}</span>
                <span className="swap-time">{new Date(swap.createdAt).toLocaleTimeString()}</span>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );

  return (
    <div className="app">
      {/* Header */}
      <header className="header">
        <div className="header-content">
          <div className="brand">
            <h1>Polkavex</h1>
            <span className="beta-badge">AI-Powered Cross-Chain</span>
          </div>
          
          <div className="connection-status">
            <div className={`status-indicator ${isConnected ? 'connected' : 'disconnected'}`}>
              <div className="status-dot"></div>
              {isConnected ? 'Connected' : 'Disconnected'}
            </div>
          </div>
        </div>
      </header>

      {/* Navigation */}
      <nav className="navigation">
        <div className="nav-content">
          <button
            onClick={() => setActiveTab('swap')}
            className={`nav-button ${activeTab === 'swap' ? 'active' : ''}`}
          >
            🔄 Swap
          </button>
          <button
            onClick={() => setActiveTab('dashboard')}
            className={`nav-button ${activeTab === 'dashboard' ? 'active' : ''}`}
          >
            📊 Dashboard
          </button>
        </div>
      </nav>

      {/* Main Content */}
      <main className="main-content">
        {activeTab === 'swap' ? renderSwapInterface() : renderDashboard()}
      </main>
    </div>
  );
}

export default App;
