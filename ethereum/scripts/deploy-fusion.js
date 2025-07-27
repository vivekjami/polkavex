const hre = require("hardhat");

async function main() {
  console.log("🚀 Deploying FusionEscrow contract...");
  
  const FusionEscrow = await hre.ethers.getContractFactory("FusionEscrow");
  const fusionEscrow = await FusionEscrow.deploy();
  
  await fusionEscrow.waitForDeployment();
  
  const contractAddress = await fusionEscrow.getAddress();
  console.log("✅ FusionEscrow deployed to:", contractAddress);
  console.log("🔗 Etherscan URL:", `https://sepolia.etherscan.io/address/${contractAddress}`);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
