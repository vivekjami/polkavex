import React, { useState, useEffect } from 'react';
import { ethers } from 'ethers';
import io from 'socket.io-client';
import { Chart as ChartJS, CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip, Legend, ArcElement, BarElement } from 'chart.js';
import { Line, Doughnut, Bar } from 'react-chartjs-2';
import './App.css';

// Register Chart.js components
ChartJS.register(CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip, Legend, ArcElement, BarElement);

// Initialize socket connection to relayer
const socket = io('http://localhost:3002');

interface SwapData {
  swapId: string;
  status: string;
  amount: string;
  token: string;
  sourceChain: string;
  targetChain: string;
  createdAt: string;
  progress?: any[];
  txHash?: string;
  estimatedTime?: string;
}

interface AnalyticsData {
  totalSwaps: number;
  totalVolume: string;
  successRate: number;
  avgTime: string;
  chartData: any;
  recentSwaps: SwapData[];
}

interface WalletState {
  isConnected: boolean;
  address: string;
  balance: string;
  chainId: number;
}

function App() {
  const [activeTab, setActiveTab] = useState<'swap' | 'dashboard'>('swap');
  const [swapData, setSwapData] = useState<SwapData | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [loading, setLoading] = useState(false);
  const [walletConnected, setWalletConnected] = useState(false);
  const [walletAddress, setWalletAddress] = useState('');
  const [analytics, setAnalytics] = useState<AnalyticsData>({
    totalSwaps: 0,
    totalVolume: '0',
    successRate: 0,
    avgTime: '0s',
    chartData: { labels: [], datasets: [] },
    recentSwaps: []
  });

  // Form state
  const [formData, setFormData] = useState({
    amount: '',
    token: 'USDC',
    sourceChain: 'ethereum',
    targetChain: 'polkadot',
    beneficiary: ''
  });

  useEffect(() => {
    // Check relayer connection
    checkRelayerConnection();
    
    // Set up socket listeners
    socket.on('connect', () => {
      console.log('Connected to Polkavex relayer');
      setIsConnected(true);
    });

    socket.on('disconnect', () => {
      setIsConnected(false);
    });

    socket.on('swap-initiated', (data: SwapData) => {
      console.log('Swap initiated:', data);
      setSwapData(data);
    });

    socket.on('swap-updated', (data: SwapData) => {
      console.log('Swap updated:', data);
      setSwapData(data);
    });

    // Auto-refresh analytics every 30 seconds
    const analyticsInterval = setInterval(() => {
      if (activeTab === 'dashboard') {
        fetchAnalytics();
      }
    }, 30000);

    return () => {
      socket.off('connect');
      socket.off('disconnect');
      socket.off('swap-initiated');
      socket.off('swap-updated');
      clearInterval(analyticsInterval);
    };
  }, [activeTab]);

  const checkRelayerConnection = async () => {
    try {
      const response = await fetch('http://localhost:3002/health');
      const data = await response.json();
      setIsConnected(data.status === 'ok');
    } catch (error) {
      console.error('Relayer connection failed:', error);
      setIsConnected(false);
    }
  };

  const initiateSwap = async () => {
    if (!formData.amount || !formData.beneficiary) {
      alert('Please fill all required fields');
      return;
    }

    setLoading(true);
    try {
      const secretHash = ethers.keccak256(ethers.toUtf8Bytes(`secret_${Date.now()}`));
      
      const response = await fetch('http://localhost:3002/api/initiate-swap', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          ...formData,
          secretHash,
          timelock: Math.floor(Date.now() / 1000) + 3600 // 1 hour
        })
      });

      const result = await response.json();
      console.log('Swap initiated:', result);
      setSwapData(result.swapDetails || result);
      
      // Fetch analytics after swap
      await fetchAnalytics();
      
    } catch (error) {
      console.error('Swap initiation failed:', error);
      alert('Failed to initiate swap. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  const fetchAnalytics = async () => {
    try {
      await fetch('http://localhost:3002/health');
      
      // Mock analytics data for demo (in production, this would come from the relayer API)
      const mockAnalytics: AnalyticsData = {
        totalSwaps: 47,
        totalVolume: '125,840 USDC',
        successRate: 98.3,
        avgTime: '2m 34s',
        recentSwaps: [
          {
            swapId: 'swap_abc123',
            status: 'completed',
            amount: '1000',
            token: 'USDC',
            sourceChain: 'ethereum',
            targetChain: 'polkadot',
            createdAt: new Date(Date.now() - 300000).toISOString(),
            txHash: '0x1234...abcd'
          },
          {
            swapId: 'swap_def456',
            status: 'pending',
            amount: '500',
            token: 'ETH',
            sourceChain: 'polkadot',
            targetChain: 'ethereum',
            createdAt: new Date(Date.now() - 600000).toISOString()
          }
        ],
        chartData: {
          labels: ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'],
          datasets: [
            {
              label: 'Daily Volume (USDC)',
              data: [12000, 19000, 3000, 5000, 2000, 3000, 20000],
              borderColor: 'rgb(75, 192, 192)',
              backgroundColor: 'rgba(75, 192, 192, 0.2)',
              tension: 0.4
            }
          ]
        }
      };
      
      setAnalytics(mockAnalytics);
    } catch (error) {
      console.error('Failed to fetch analytics:', error);
    }
  };

  useEffect(() => {
    if (activeTab === 'dashboard') {
      fetchAnalytics();
    }
  }, [activeTab]);

  const renderSwapInterface = () => (
    <div className="swap-container">
      <div className="swap-header">
        <h2>Cross-Chain Swap</h2>
        <p>Bridge assets between Ethereum and Polkadot with AI-optimized routing</p>
      </div>

      <div className="swap-form">
        <div className="input-row">
          <div className="input-group">
            <label>Amount</label>
            <input
              type="number"
              value={formData.amount}
              onChange={(e) => setFormData({ ...formData, amount: e.target.value })}
              placeholder="0.00"
            />
          </div>
          <div className="input-group">
            <label>Token</label>
            <select
              value={formData.token}
              onChange={(e) => setFormData({ ...formData, token: e.target.value })}
              title="Select token type"
            >
              <option value="USDC">USDC</option>
              <option value="ETH">ETH</option>
              <option value="DOT">DOT</option>
            </select>
          </div>
        </div>

        <div className="input-row">
          <div className="input-group">
            <label>From Chain</label>
            <select
              value={formData.sourceChain}
              onChange={(e) => setFormData({ ...formData, sourceChain: e.target.value })}
              title="Select source blockchain"
            >
              <option value="ethereum">Ethereum</option>
              <option value="polkadot">Polkadot</option>
            </select>
          </div>
          <div className="input-group">
            <label>To Chain</label>
            <select
              value={formData.targetChain}
              onChange={(e) => setFormData({ ...formData, targetChain: e.target.value })}
              title="Select target blockchain"
            >
              <option value="polkadot">Polkadot</option>
              <option value="ethereum">Ethereum</option>
            </select>
          </div>
        </div>

        <div className="input-group full-width">
          <label>Recipient Address</label>
          <input
            type="text"
            value={formData.beneficiary}
            onChange={(e) => setFormData({ ...formData, beneficiary: e.target.value })}
            placeholder="0x... or 5..."
          />
        </div>

        <button
          onClick={initiateSwap}
          disabled={loading || !isConnected}
          className={`swap-button ${loading || !isConnected ? 'disabled' : ''}`}
        >
          {loading ? 'Initiating Swap...' : 'Initiate Cross-Chain Swap'}
        </button>
      </div>

      {swapData && (
        <div className="swap-status">
          <h3>Swap Status</h3>
          <div className="status-details">
            <p><span className="label">Swap ID:</span> {swapData.swapId}</p>
            <p><span className="label">Status:</span> 
              <span className={`status-badge ${swapData.status}`}>
                {swapData.status}
              </span>
            </p>
            <p><span className="label">Amount:</span> {swapData.amount} {swapData.token}</p>
            <p><span className="label">Route:</span> {swapData.sourceChain} → {swapData.targetChain}</p>
            {swapData.txHash && (
              <p><span className="label">Tx Hash:</span> 
                <a href={`https://sepolia.etherscan.io/tx/${swapData.txHash}`} target="_blank" rel="noopener noreferrer">
                  {swapData.txHash.substring(0, 10)}...{swapData.txHash.substring(swapData.txHash.length - 8)}
                </a>
              </p>
            )}
            {swapData.estimatedTime && (
              <p><span className="label">Est. Time:</span> {swapData.estimatedTime}</p>
            )}
          </div>
          
          {/* Progress indicator */}
          <div className="progress-container">
            <div className="progress-bar">
              <div className={`progress-fill ${swapData.status}`}></div>
            </div>
            <div className="progress-steps">
              <div className={`step ${['pending', 'processing', 'completed'].includes(swapData.status) ? 'active' : ''}`}>
                Initiated
              </div>
              <div className={`step ${['processing', 'completed'].includes(swapData.status) ? 'active' : ''}`}>
                Processing
              </div>
              <div className={`step ${swapData.status === 'completed' ? 'active' : ''}`}>
                Completed
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );

  const renderDashboard = () => (
    <div className="dashboard-container">
      <div className="dashboard-header">
        <h2>Analytics Dashboard</h2>
        <p>Real-time insights into cross-chain swap performance</p>
      </div>

      <div className="metrics-grid">
        <div className="metric-card">
          <h3>Total Swaps</h3>
          <p className="metric-value">{analytics.totalSwaps}</p>
        </div>
        <div className="metric-card">
          <h3>Total Volume</h3>
          <p className="metric-value">{analytics.totalVolume}</p>
        </div>
        <div className="metric-card">
          <h3>Success Rate</h3>
          <p className="metric-value success">{analytics.successRate}%</p>
        </div>
        <div className="metric-card">
          <h3>Avg Time</h3>
          <p className="metric-value">{analytics.avgTime}</p>
        </div>
      </div>

      <div className="charts-grid">
        <div className="chart-card">
          <h3>Daily Volume</h3>
          <div className="chart-container">
            <Line
              data={analytics.chartData}
              options={{
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                  legend: { position: 'top' as const },
                  title: { display: false }
                }
              }}
            />
          </div>
        </div>
        
        <div className="chart-card">
          <h3>Chain Distribution</h3>
          <div className="chart-container">
            <Doughnut
              data={{
                labels: ['Ethereum → Polkadot', 'Polkadot → Ethereum', 'Acala Route', 'Moonbeam Route'],
                datasets: [{
                  data: [35, 25, 25, 15],
                  backgroundColor: ['#3B82F6', '#EF4444', '#10B981', '#F59E0B'],
                  borderWidth: 0
                }]
              }}
              options={{
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                  legend: { position: 'bottom' as const }
                }
              }}
            />
          </div>
        </div>
      </div>

      <div className="recent-activity">
        <h3>Recent Swaps</h3>
        <div className="activity-list">
          {analytics.recentSwaps.map((swap, index) => (
            <div key={index} className="activity-item">
              <div className="activity-info">
                <span className="swap-id">{swap.swapId}</span>
                <span className="swap-route">{swap.sourceChain} → {swap.targetChain}</span>
              </div>
              <div className="activity-details">
                <span className="amount">{swap.amount} {swap.token}</span>
                <span className={`status-badge ${swap.status}`}>{swap.status}</span>
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
            Swap
          </button>
          <button
            onClick={() => setActiveTab('dashboard')}
            className={`nav-button ${activeTab === 'dashboard' ? 'active' : ''}`}
          >
            Dashboard
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
