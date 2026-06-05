// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title PNKTheosisOracle
 * @dev Oracle providing Theosis scores (0-1000) for PNK jurors.
 * In a real scenario, this would be updated by the Arkhe WormGraph/ZkAGI.
 */
contract PNKTheosisOracle {
    address public owner;

    // Mapping from juror address to their Theosis score (0 to 1000)
    mapping(address => uint256) public theosisScores;

    event TheosisScoreUpdated(address indexed juror, uint256 oldScore, uint256 newScore);

    constructor() {
        owner = msg.sender;
    }

    modifier onlyOwner() {
        require(msg.sender == owner, "Not owner");
        _;
    }

    /**
     * @dev Updates the Theosis score for a juror.
     * @param juror Address of the juror.
     * @param score New Theosis score (must be <= 1000).
     */
    function updateScore(address juror, uint256 score) external onlyOwner {
        require(score <= 1000, "Score exceeds maximum (1000)");
        uint256 oldScore = theosisScores[juror];
        theosisScores[juror] = score;
        emit TheosisScoreUpdated(juror, oldScore, score);
    }

    /**
     * @dev Retrieves the Theosis score for a juror.
     * @param juror Address of the juror.
     * @return Theosis score (0-1000).
     */
    function getTheosisScore(address juror) external view returns (uint256) {
        return theosisScores[juror];
    }
}
