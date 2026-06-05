import { expect } from "chai";
import { ethers } from "hardhat";

describe("CathedralKlerosBridgeWithVoting", function () {
  let bridge: any;
  let oracle: any;
  let mockVea: any;
  let owner: any;
  let juror1: any;
  let juror2: any;

  before(async function () {
    [owner, juror1, juror2] = await ethers.getSigners();

    // Deploy Mock VeaInbox
    const MockVeaInbox = await ethers.getContractFactory("MockVeaInbox");
    mockVea = await MockVeaInbox.deploy();
    await mockVea.deployed();

    // Deploy PNKTheosisOracle
    const PNKTheosisOracle = await ethers.getContractFactory("PNKTheosisOracle");
    oracle = await PNKTheosisOracle.deploy();
    await oracle.deployed();

    // Deploy CathedralKlerosBridgeWithVoting
    const rbbTargetContract = "0x0000000000000000000000000000000000000003"; // Dummy target
    const Bridge = await ethers.getContractFactory("CathedralKlerosBridgeWithVoting");
    bridge = await Bridge.deploy(mockVea.address, rbbTargetContract, oracle.address);
    await bridge.deployed();
  });

  it("should deploy correctly and set initial values", async function () {
    expect(await bridge.veaInbox()).to.equal(mockVea.address);
    expect(await bridge.theosisOracle()).to.equal(oracle.address);
    expect(await bridge.owner()).to.equal(owner.address);
  });

  it("should update Theosis scores", async function () {
    await oracle.updateScore(juror1.address, 200);
    await oracle.updateScore(juror2.address, 800);

    expect(await oracle.getTheosisScore(juror1.address)).to.equal(200);
    expect(await oracle.getTheosisScore(juror2.address)).to.equal(800);
  });

  it("should calculate voting weights correctly", async function () {
    // 1000 + 200 * 4 = 1800
    const weight1 = await bridge.getVotingWeight(juror1.address);
    expect(weight1).to.equal(1800);

    // 1000 + 800 * 4 = 4200
    const weight2 = await bridge.getVotingWeight(juror2.address);
    expect(weight2).to.equal(4200);
  });

  it("should cast weighted votes and prevent double voting", async function () {
    const disputeId = 1;
    await bridge.createDispute(disputeId);

    await bridge.connect(juror1).castWeightedVote(disputeId, 1);
    let dispute = await bridge.disputes(disputeId);
    expect(dispute.totalVotes).to.equal(1800);
    expect(dispute.winningChoice).to.equal(1);

    // Try voting again
    await expect(
        bridge.connect(juror1).castWeightedVote(disputeId, 2)
    ).to.be.revertedWith("Juror already voted");

    await bridge.connect(juror2).castWeightedVote(disputeId, 2);
    dispute = await bridge.disputes(disputeId);
    expect(dispute.totalVotes).to.equal(1800 + 4200);
    expect(dispute.winningChoice).to.equal(2); // juror2 has more weight
  });

  it("should relay decision via Vea Inbox using winning ruling", async function () {
    const disputeId = 1;
    await expect(bridge.relayDecisionToRBB(disputeId))
      .to.emit(mockVea, "MessageSent"); // Adjust according to MockVea events if needed

    const dispute = await bridge.disputes(disputeId);
    expect(dispute.active).to.be.false;
  });
});
