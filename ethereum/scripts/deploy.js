const { ethers } = require("hardhat");

async function main() {
  console.log("🚀 Deploying PolkavexEscrow contract...");

  const PolkavexEscrow = await ethers.getContractFactory("PolkavexEscrow");
  const escrow = await PolkavexEscrow.deploy();

  await escrow.deployed();

  console.log("✅ PolkavexEscrow deployed to:", escrow.address);
  console.log("📋 Transaction hash:", escrow.deployTransaction.hash);
  
  // Save deployment info
  const deploymentInfo = {
    address: escrow.address,
    transactionHash: escrow.deployTransaction.hash,
    network: network.name,
    deployedAt: new Date().toISOString()
  };

  console.log("💾 Deployment Info:", JSON.stringify(deploymentInfo, null, 2));
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error("❌ Deployment failed:", error);
    process.exit(1);
  });
