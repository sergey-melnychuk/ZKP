// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IVerifier {
    function verifyProof(
        uint[2] calldata _pA,
        uint[2][2] calldata _pB,
        uint[2] calldata _pC,
        uint[2] calldata _pubSignals
    ) external view returns (bool);
}

interface IMerkleTree {
    function root() external view returns (uint256);
}

contract MerkleProofVerifier {
    IVerifier public verifier;
    IMerkleTree public tree;
    
    mapping(uint256 => bool) public nullifierUsed;
    
    event ProofVerified(address indexed user, uint256 nullifier);
    
    constructor(address _verifier, address _tree) {
        verifier = IVerifier(_verifier);
        tree = IMerkleTree(_tree);
    }
    
    function submitProof(
        uint[2] calldata _pA,
        uint[2][2] calldata _pB,
        uint[2] calldata _pC,
        uint256 nullifier
    ) external returns (bool) {
        require(!nullifierUsed[nullifier], "Nullifier already used");
        
        uint256 currentRoot = tree.root();
        uint[2] memory pubSignals = [currentRoot, nullifier];
        
        bool valid = verifier.verifyProof(_pA, _pB, _pC, pubSignals);
        require(valid, "Invalid proof");
        
        nullifierUsed[nullifier] = true;
        emit ProofVerified(msg.sender, nullifier);
        
        return true;
    }
}
