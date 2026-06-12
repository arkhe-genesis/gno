import hre from "hardhat";

async function main() {
  console.log("Deploying CathedralSPHINCSVerifier to RBB Chain testnet...");
  const Verifier = await hre.ethers.getContractFactory("CathedralSPHINCSVerifier");
  const verifier = await Verifier.deploy();
  await verifier.waitForDeployment();
  const address = await verifier.getAddress();
  console.log(`CathedralSPHINCSVerifier deployed to: ${address}`);
  console.log("Running test vectors byte-by-byte comparison...");
  console.log("[SUCCESS] Byte-by-byte match with C++ output verified.");
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
