// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./TheosisWeightedVoting.sol";

/**
 * @title CathedralKlerosBridgeWithVoting
 * @dev Integration bridge that allows Cathedral AI entities to interact with Kleros courts,
 * specifically utilizing Theosis-weighted voting for jurors in designated Cathedral Courts.
 */
contract CathedralKlerosBridgeWithVoting {
    address public owner;
    TheosisWeightedVoting public votingModule;

    // Dispute ID to Court ID
    mapping(uint256 => uint256) public disputeToCourt;

    // Court ID to bool indicating if it uses Theosis voting
    mapping(uint256 => bool) public isCathedralCourt;

    event BridgeInitialized(address votingModule);
    event CourtStatusUpdated(uint256 courtId, bool usesTheosis);
    event DisputeCreated(uint256 disputeId, uint256 courtId, string metadata);

    modifier onlyOwner() {
        require(msg.sender == owner, "Only owner");
        _;
    }

    constructor(address _votingModule) {
        owner = msg.sender;
        votingModule = TheosisWeightedVoting(_votingModule);
        emit BridgeInitialized(_votingModule);
    }

    function setCourtStatus(uint256 courtId, bool usesTheosis) external onlyOwner {
        isCathedralCourt[courtId] = usesTheosis;
        emit CourtStatusUpdated(courtId, usesTheosis);
    }

    /**
     * @dev Mock function to create a dispute in the Cathedral ecosystem.
     * In reality, this would interface with Kleros IArbitrator.
     */
    function createCathedralDispute(uint256 disputeId, uint256 courtId, string calldata metadata) external onlyOwner {
        disputeToCourt[disputeId] = courtId;
        emit DisputeCreated(disputeId, courtId, metadata);
    }

    /**
     * @dev Resolves the effective voting power for a juror in a specific dispute.
     */
    function getJurorVotingPower(uint256 disputeId, address juror, uint256 basePower) external view returns (uint256) {
        uint256 courtId = disputeToCourt[disputeId];

        if (isCathedralCourt[courtId]) {
            return votingModule.getEffectiveWeight(juror, basePower);
        }

        return basePower; // Standard Kleros voting power
    }
}
