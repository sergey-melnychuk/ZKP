#!/bin/sh
circom sudoku.circom --r1cs --wasm --sym
snarkjs r1cs info sudoku.r1cs

cat > input.json << EOF
{
  "solution": [
    5,3,4,6,7,8,9,1,2,
    6,7,2,1,9,5,3,4,8,
    1,9,8,3,4,2,5,6,7,
    8,5,9,7,6,1,4,2,3,
    4,2,6,8,5,3,7,9,1,
    7,1,3,9,2,4,8,5,6,
    9,6,1,5,3,7,2,8,4,
    2,8,7,4,1,9,6,3,5,
    3,4,5,2,8,6,1,7,9
  ],
  "puzzle": [
    5,3,0,0,7,0,0,0,0,
    6,0,0,1,9,5,0,0,0,
    0,9,8,0,0,0,0,6,0,
    8,0,0,0,6,0,0,0,3,
    4,0,0,8,0,3,0,0,1,
    7,0,0,0,2,0,0,0,6,
    0,6,0,0,0,0,2,8,0,
    0,0,0,4,1,9,0,0,5,
    0,0,0,0,8,0,0,7,9
  ]
}
EOF

node sudoku_js/generate_witness.js sudoku_js/sudoku.wasm input.json witness.wtns

snarkjs powersoftau new bn128 12 pot12_0000.ptau -v
snarkjs powersoftau contribute pot12_0000.ptau pot12_0001.ptau --name="1st contribution" -v -e="random"
snarkjs powersoftau prepare phase2 pot12_0001.ptau pot12_final.ptau -v

snarkjs groth16 setup sudoku.r1cs pot12_final.ptau sudoku_0000.zkey
snarkjs zkey contribute sudoku_0000.zkey sudoku_final.zkey --name="1st Contributor" -v -e="random"
snarkjs zkey export verificationkey sudoku_final.zkey verification_key.json

snarkjs groth16 prove sudoku_final.zkey witness.wtns proof.json public.json

snarkjs groth16 verify verification_key.json public.json proof.json

snarkjs zkey export solidityverifier sudoku_final.zkey verifier.sol
