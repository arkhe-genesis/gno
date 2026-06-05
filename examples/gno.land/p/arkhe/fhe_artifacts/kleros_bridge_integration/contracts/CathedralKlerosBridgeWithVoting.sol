// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title CathedralKlerosBridgeWithVoting
 * @dev Bridges Kleros Arbitrum dispute resolution with Arkhe Cathedral (RBB)
 * Integrates Vea Relay for cross-chain messaging and Theosis-weighted voting.
 */

interface IVeaInbox {
    function sendMessage(address _to, bytes4 _fnSelector, bytes memory _data) external returns (uint64);
}

interface IPNKTheosisOracle {
    function getTheosisScore(address juror) external view returns (uint256);
}

contract CathedralKlerosBridgeWithVoting {
    address public veaInbox;
    address public rbbTargetContract;
    IPNKTheosisOracle public theosisOracle;
    address public owner;

    uint256 public constant BASE_WEIGHT = 1000;
    uint256 public constant MAX_THEOSIS_MULTIPLIER = 5000; // Max 5x voting power

    event DisputeRelayed(uint256 indexed disputeId, uint256 winningRuling, uint256 totalVotes);
    event VoteCast(uint256 indexed disputeId, address indexed juror, uint256 vote, uint256 theosisWeight);
    event DisputeCreated(uint256 indexed disputeId);

    struct DisputeInfo {
        bool active;
        uint256 totalVotes;
        uint256 winningChoice;
        uint256 maxVotesForChoice;
        mapping(uint256 => uint256) voteTally; // Maps ruling option to total weight
        mapping(address => bool) hasVoted;     // Prevents double voting
    }

    mapping(uint256 => DisputeInfo) public disputes;

    modifier onlyOwner() {
        require(msg.sender == owner, "Not owner");
        _;
    }

    constructor(address _veaInbox, address _rbbTargetContract, address _theosisOracle) {
        veaInbox = _veaInbox;
        rbbTargetContract = _rbbTargetContract;
        theosisOracle = IPNKTheosisOracle(_theosisOracle);
        owner = msg.sender;
    }

    /**
     * @dev Calculates the voting weight for a given juror based on their Theosis score.
     * Theosis score is expected to be between 0 and 1000.
     */
    function getVotingWeight(address juror) public view returns (uint256) {
        uint256 theosisScore = theosisOracle.getTheosisScore(juror);
        // Map Theosis score (0-1000) to a multiplier (1x to 5x)
        // Weight = BASE_WEIGHT + (theosisScore * (MAX_THEOSIS_MULTIPLIER - BASE_WEIGHT) / 1000)
        uint256 extraWeight = (theosisScore * (MAX_THEOSIS_MULTIPLIER - BASE_WEIGHT)) / 1000;
        return BASE_WEIGHT + extraWeight;
    }

    /**
     * @dev Casts a vote in a dispute, weighted by Theosis score.
     */
    function castWeightedVote(uint256 disputeId, uint256 vote) external {
        DisputeInfo storage dispute = disputes[disputeId];
        require(dispute.active, "Dispute not active");
        require(!dispute.hasVoted[msg.sender], "Juror already voted");

        uint256 weight = getVotingWeight(msg.sender);

        dispute.hasVoted[msg.sender] = true;
        dispute.totalVotes += weight;
        dispute.voteTally[vote] += weight;

        // Keep track of the winning choice
        if (dispute.voteTally[vote] > dispute.maxVotesForChoice) {
            dispute.maxVotesForChoice = dispute.voteTally[vote];
            dispute.winningChoice = vote;
        }

        emit VoteCast(disputeId, msg.sender, vote, weight);
    }

    /**
     * @dev Resolves the dispute based on the tallied votes and relays the decision to RBB.
     */
    function relayDecisionToRBB(uint256 disputeId) external {
        DisputeInfo storage dispute = disputes[disputeId];
        require(dispute.active, "Dispute not active");
        // In a real Kleros integration, this would be restricted to a specific period or condition
        require(msg.sender == owner, "Only owner can finalize currently");

        uint256 ruling = dispute.winningChoice;

        // Encode the payload for the RBB target contract
        bytes4 selector = bytes4(keccak256("executeRuling(uint256,uint256)"));
        bytes memory data = abi.encode(disputeId, ruling);

        IVeaInbox(veaInbox).sendMessage(rbbTargetContract, selector, data);

        dispute.active = false;

        emit DisputeRelayed(disputeId, ruling, dispute.totalVotes);
    }

    function createDispute(uint256 disputeId) external onlyOwner {
        DisputeInfo storage dispute = disputes[disputeId];
        require(!dispute.active, "Dispute already active");
        dispute.active = true;
        dispute.totalVotes = 0;
        emit DisputeCreated(disputeId);
    }

    function getVoteTally(uint256 disputeId, uint256 vote) external view returns (uint256) {
        return disputes[disputeId].voteTally[vote];
    }

    function hasJurorVoted(uint256 disputeId, address juror) external view returns (bool) {
        return disputes[disputeId].hasVoted[juror];
    }
}
