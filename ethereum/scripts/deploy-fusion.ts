import { ethers } from "hardhat";

async function main() {
  console.log("🚀 Deploying FusionEscrow contract...");
  
  // Get the contract factory
  const FusionEscrow = await ethers.getContractFactory("FusionEscrow");
  
  // Deploy the contract
  console.log("📋 Deploying with account:", await (await ethers.getSigners())[0].getAddress());
  
  const fusionEscrow = await FusionEscrow.deploy();
  await fusionEscrow.waitForDeployment();
  
  const contractAddress = await fusionEscrow.getAddress();
  
  console.log("✅ FusionEscrow deployed to:", contractAddress);
  console.log("🔗 Transaction hash:", fusionEscrow.deploymentTransaction()?.hash);
  
  // Wait a few blocks for Etherscan indexing
  console.log("⏳ Waiting for block confirmations...");
  await fusionEscrow.deploymentTransaction()?.wait(2);
  
  console.log("🎉 Deployment complete!");
  console.log("📊 Contract details:");
  console.log("  - Address:", contractAddress);
  console.log("  - Network: Sepolia Testnet");
  console.log("  - Etherscan:", `https://sepolia.etherscan.io/address/${contractAddress}`);
  
  // Save deployment info for future reference
  const deploymentInfo = {
    address: contractAddress,
    network: "sepolia",
    deployedAt: new Date().toISOString(),
    deployer: await (await ethers.getSigners())[0].getAddress(),
    txHash: fusionEscrow.deploymentTransaction()?.hash
  };
  
  console.log("\n📝 Save this deployment info:");
  console.log(JSON.stringify(deploymentInfo, null, 2));
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error("❌ Deployment failed:", error);
    process.exit(1);
  });
