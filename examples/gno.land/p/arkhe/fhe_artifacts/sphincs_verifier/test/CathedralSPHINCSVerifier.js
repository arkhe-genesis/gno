import { expect } from "chai";
import hre from "hardhat";

describe("CathedralSPHINCSVerifier", function () {
  it("Should estimate gas correctly without grinding", async function () {
    const Verifier = await hre.ethers.getContractFactory("CathedralSPHINCSVerifier");
    const verifier = await Verifier.deploy();

    const message = hre.ethers.randomBytes(32);
    const signature = hre.ethers.randomBytes(3952);
    const publicKeyRoot = hre.ethers.randomBytes(32);

    const gasEstimate = await verifier.verifySPHINCS.estimateGas(message, signature, publicKeyRoot);
    console.log("Estimated Gas:", gasEstimate.toString());
  });
});
