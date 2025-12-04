#!/bin/bash

echo "Setup ceremony..."
snarkjs powersoftau new bn128 12 pot12_0000.ptau -v
snarkjs powersoftau contribute pot12_0000.ptau pot12_0001.ptau --name="First" -v -e="random"
snarkjs powersoftau prepare phase2 pot12_0001.ptau pot12_final.ptau -v

echo "Generating keys..."
snarkjs groth16 setup merkle.r1cs pot12_final.ptau merkle_0000.zkey
snarkjs zkey contribute merkle_0000.zkey merkle_final.zkey --name="Contributor" -v -e="random"
snarkjs zkey export verificationkey merkle_final.zkey verification_key.json

echo "Generating Solidity verifier..."
snarkjs zkey export solidityverifier merkle_final.zkey contracts/Verifier.sol

echo "✅ Done!"
