// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title PNKTheosisOracle
 * @dev Oracle that bridges the Cathedral's Theosis score for jurors into the EVM.
 * Updates are authenticated by the Cathedral's ZK-proofs or authorized updater.
 */
contract PNKTheosisOracle {
    address public owner;
    address public updater;

    // Juror address => Theosis Score (0 to 10000, representing 0.0 to 1.0)
    mapping(address => uint256) public jurorTheosis;

    event TheosisUpdated(address indexed juror, uint256 newTheosis, uint256 timestamp);

    modifier onlyOwner() {
        require(msg.sender == owner, "Only owner");
        _;
    }

    modifier onlyUpdater() {
        require(msg.sender == updater || msg.sender == owner, "Only updater");
        _;
    }

    constructor() {
        owner = msg.sender;
        updater = msg.sender;
    }

    function setUpdater(address _updater) external onlyOwner {
        updater = _updater;
    }

    /**
     * @dev Updates the Theosis score for a juror.
     * @param juror The address of the juror.
     * @param theosis The new Theosis score (0-10000).
     */
    function updateTheosis(address juror, uint256 theosis) external onlyUpdater {
        require(theosis <= 10000, "Theosis score out of bounds");
        jurorTheosis[juror] = theosis;
        emit TheosisUpdated(juror, theosis, block.timestamp);
    }

    /**
     * @dev Batch updates Theosis scores.
     */
    function batchUpdateTheosis(address[] calldata jurors, uint256[] calldata theosises) external onlyUpdater {
        require(jurors.length == theosises.length, "Length mismatch");
        for (uint256 i = 0; i < jurors.length; i++) {
            require(theosises[i] <= 10000, "Theosis score out of bounds");
            jurorTheosis[jurors[i]] = theosises[i];
            emit TheosisUpdated(jurors[i], theosises[i], block.timestamp);
        }
    }

    function getTheosis(address juror) external view returns (uint256) {
        return jurorTheosis[juror];
    }
}
