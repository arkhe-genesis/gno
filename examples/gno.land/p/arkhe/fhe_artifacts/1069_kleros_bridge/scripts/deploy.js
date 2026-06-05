const hre = require("hardhat");

async function main() {
  console.log("Starting Hardhat deployment of Cathedral Kleros Bridge contracts...");

  // 1. Deploy PNKTheosisOracle
  const PNKTheosisOracle = await hre.ethers.getContractFactory("PNKTheosisOracle");
  const oracle = await PNKTheosisOracle.deploy();
  await oracle.deployed();
  console.log(`PNKTheosisOracle deployed to: ${oracle.address}`);

  // 2. Deploy TheosisWeightedVoting
  const theosisMultiplier = 50; // Max 50% extra voting weight based on Theosis
  const TheosisWeightedVoting = await hre.ethers.getContractFactory("TheosisWeightedVoting");
  const voting = await TheosisWeightedVoting.deploy(oracle.address, theosisMultiplier);
  await voting.deployed();
  console.log(`TheosisWeightedVoting deployed to: ${voting.address}`);

  // 3. Deploy CathedralKlerosBridgeWithVoting
  const CathedralKlerosBridgeWithVoting = await hre.ethers.getContractFactory("CathedralKlerosBridgeWithVoting");
  const bridge = await CathedralKlerosBridgeWithVoting.deploy(voting.address);
  await bridge.deployed();
  console.log(`CathedralKlerosBridgeWithVoting deployed to: ${bridge.address}`);

  // 4. Initialize Cathedral Court settings (e.g., Court 1 is a Cathedral Court)
  console.log("Setting up initial court configurations...");
  const tx = await bridge.setCourtStatus(1, true);
  await tx.wait();
  console.log("Court 1 marked as Cathedral Court (uses Theosis voting).");

  console.log("Deployment complete.");
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
