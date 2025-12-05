pragma circom 2.0.0;

include "circomlib/circuits/poseidon.circom";

template MerkleProof() {
    signal input secret;
    signal input siblings[8];
    signal input pathIndices[8];
    
    signal input root;
    signal input nullifier;
    
    // Compute leaf
    component leafHasher = Poseidon(1);
    leafHasher.inputs[0] <== secret;
    
    // Declare all signals outside loop
    component hashers[8];
    signal hashes[9];
    signal lefts[8];
    signal rights[8];
    
    hashes[0] <== leafHasher.out;
    
    for (var i = 0; i < 8; i++) {
        hashers[i] = Poseidon(2);
        
        // Compute left and right based on pathIndices[i]
        lefts[i] <== hashes[i] - pathIndices[i] * (hashes[i] - siblings[i]);
        rights[i] <== siblings[i] + pathIndices[i] * (hashes[i] - siblings[i]);
        
        hashers[i].inputs[0] <== lefts[i];
        hashers[i].inputs[1] <== rights[i];
        hashes[i + 1] <== hashers[i].out;
    }
    
    // Verify root
    root === hashes[8];
    
    // Verify nullifier
    component nullHasher = Poseidon(1);
    nullHasher.inputs[0] <== secret;
    nullifier === nullHasher.out;
}

component main {public [root, nullifier]} = MerkleProof();
