import { HardhatUserConfig } from "hardhat/config";
import "@nomicfoundation/hardhat-toolbox";
import "dotenv/config";

const config: HardhatUserConfig = {
  solidity: {
    version: "0.8.28",
    settings: {
      optimizer: {
        enabled: true,
        runs: 200,
      },
    },
  },
  networks: {
    sepolia: {
      url: process.env.ETHEREUM_RPC_URL || "https://sepolia.infura.io/v3/YOUR_INFURA_KEY",
      accounts: process.env.ETHEREUM_PRIVATE_KEY ? [process.env.ETHEREUM_PRIVATE_KEY] : [],
      gasPrice: "auto",
    },
    hardhat: {
      forking: {
        url: process.env.ETHEREUM_RPC_URL || "https://sepolia.infura.io/v3/YOUR_INFURA_KEY",
        enabled: false,
      },
    },
  },
  etherscan: {
    apiKey: process.env.ETHERSCAN_API_KEY,
  },
  gasReporter: {
    enabled: true,
    currency: "USD",
  },
};

export default config;
