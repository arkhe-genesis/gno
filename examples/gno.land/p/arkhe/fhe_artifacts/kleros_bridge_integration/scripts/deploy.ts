import { ethers } from "hardhat";

async function main() {
  const [deployer] = await ethers.getSigners();
  console.log("Deploying contracts with the account:", deployer.address);

  // In production, use the real Vea Inbox address on Arbitrum
  const veaInboxAddress = process.env.VEA_INBOX_ADDRESS || "0x0000000000000000000000000000000000000000";
  const rbbTargetContract = process.env.RBB_TARGET_ADDRESS || "0x0000000000000000000000000000000000000000";

  // Deploy Oracle
  const PNKTheosisOracle = await ethers.getContractFactory("PNKTheosisOracle");
  const oracle = await PNKTheosisOracle.deploy();
  await oracle.deployed();
  console.log("PNKTheosisOracle deployed to:", oracle.address);

  // Deploy Bridge
  const Bridge = await ethers.getContractFactory("CathedralKlerosBridgeWithVoting");
  const bridge = await Bridge.deploy(veaInboxAddress, rbbTargetContract, oracle.address);
  await bridge.deployed();
  console.log("CathedralKlerosBridgeWithVoting deployed to:", bridge.address);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
