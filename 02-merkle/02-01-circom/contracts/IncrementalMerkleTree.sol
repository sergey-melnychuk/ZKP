// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "./PoseidonT3.sol";

contract IncrementalMerkleTree {
    uint32 public constant DEPTH = 4;
    uint32 public constant MAX_LEAVES = 16;
    
    uint32 public nextIndex = 0;
    uint256 public root;
    
    uint256[DEPTH] private filledSubtrees;
    uint256[DEPTH] private zeros;
    
    mapping(uint32 => uint256) private leaves;
    
    event LeafInserted(uint256 index, uint256 root);
    
    constructor() {
        zeros[0] = 0;
        for (uint32 i = 1; i < DEPTH; i++) {
            uint256[2] memory inputs;
            inputs[0] = zeros[i-1];
            inputs[1] = zeros[i-1];
            zeros[i] = PoseidonT3.hash(inputs);
        }
        root = zeros[DEPTH - 1];
    }
    
    function insert(uint256 commitment) external returns (uint32) {
        require(nextIndex < MAX_LEAVES, "Tree full");
        
        uint32 index = nextIndex++;
        leaves[index] = commitment;
        
        uint256 currentHash = commitment;
        uint32 currentIndex = index;
        
        for (uint32 i = 0; i < DEPTH; i++) {
            uint256[2] memory inputs;
            
            if (currentIndex % 2 == 0) {
                filledSubtrees[i] = currentHash;
                inputs[0] = currentHash;
                inputs[1] = zeros[i];
            } else {
                inputs[0] = filledSubtrees[i];
                inputs[1] = currentHash;
            }
            
            currentHash = PoseidonT3.hash(inputs);
            currentIndex /= 2;
        }
        
        root = currentHash;
        emit LeafInserted(index, root);
        return index;
    }
    
    function getPath(uint32 index) external view returns (
        uint256[DEPTH] memory siblings,
        uint8[DEPTH] memory pathIndices
    ) {
        require(index < nextIndex, "Leaf not found");
        
        uint32 currentIndex = index;
        
        for (uint32 level = 0; level < DEPTH; level++) {
            bool isLeft = currentIndex % 2 == 0;
            
            if (isLeft) {
                uint32 siblingIndex = currentIndex + 1;
                if (siblingIndex < nextIndex) {
                    siblings[level] = leaves[siblingIndex];
                } else {
                    siblings[level] = zeros[level];
                }
                pathIndices[level] = 0;
            } else {
                siblings[level] = filledSubtrees[level];
                pathIndices[level] = 1;
            }
            
            currentIndex /= 2;
        }
        
        return (siblings, pathIndices);
    }
}
