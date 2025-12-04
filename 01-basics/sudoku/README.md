# 1. Create directory
mkdir sudoku-zk && cd sudoku-zk

# 2. Save the code below to sudoku.circom
```
cat > sudoku.circom << EOF
pragma circom 2.0.0;

// Helper template to check that number is in range [1..9]
template InRange() {
    signal input in;
    signal output out;
    
    signal temp[8];
    
    temp[0] <== in - 1;
    temp[1] <== temp[0] * (in - 2);
    temp[2] <== temp[1] * (in - 3);
    temp[3] <== temp[2] * (in - 4);
    temp[4] <== temp[3] * (in - 5);
    temp[5] <== temp[4] * (in - 6);
    temp[6] <== temp[5] * (in - 7);
    temp[7] <== temp[6] * (in - 8);
    out <== temp[7] * (in - 9);
    
    out === 0;
}

template Sudoku() {
    signal input solution[81];  // Full solution (private)
    signal input puzzle[81];    // Initial puzzle (public)
    
    // 1. Range check: all numbers must be [1..9]
    component rangeCheck[81];
    for (var i = 0; i < 81; i++) {
        rangeCheck[i] = InRange();
        rangeCheck[i].in <== solution[i];
    }
    
    // 2. Check that solution matches puzzle (clues)
    for (var i = 0; i < 81; i++) {
        puzzle[i] * (solution[i] - puzzle[i]) === 0;
    }
    
    // 3. Row constraints: sum = 45
    for (var row = 0; row < 9; row++) {
        var sum = 0;
        for (var col = 0; col < 9; col++) {
            sum += solution[row * 9 + col];
        }
        sum === 45;
    }
    
    // 4. Column constraints: sum = 45
    for (var col = 0; col < 9; col++) {
        var sum = 0;
        for (var row = 0; row < 9; row++) {
            sum += solution[row * 9 + col];
        }
        sum === 45;
    }
    
    // 5. 3x3 Block constraints: sum = 45
    for (var blockRow = 0; blockRow < 3; blockRow++) {
        for (var blockCol = 0; blockCol < 3; blockCol++) {
            var sum = 0;
            for (var i = 0; i < 3; i++) {
                for (var j = 0; j < 3; j++) {
                    var row = blockRow * 3 + i;
                    var col = blockCol * 3 + j;
                    sum += solution[row * 9 + col];
                }
            }
            sum === 45;
        }
    }
}
EOF
```

# 3. Compile circuit
circom sudoku.circom --r1cs --wasm --sym

# 4. Show circuit info
snarkjs r1cs info sudoku.r1cs
# Number of constraints & public/private inputs

# 5. Generate witness from input.json
```
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
```

node sudoku_js/generate_witness.js sudoku_js/sudoku.wasm input.json witness.wtns

# 6. Powers of Tau ceremony (trusted setup) - performed once
snarkjs powersoftau new bn128 12 pot12_0000.ptau -v
snarkjs powersoftau contribute pot12_0000.ptau pot12_0001.ptau --name="1st contribution" -v
snarkjs powersoftau prepare phase2 pot12_0001.ptau pot12_final.ptau -v

# 7. Generate proving key & verification key
snarkjs groth16 setup sudoku.r1cs pot12_final.ptau sudoku_0000.zkey
snarkjs zkey contribute sudoku_0000.zkey sudoku_final.zkey --name="1st Contributor" -v
snarkjs zkey export verificationkey sudoku_final.zkey verification_key.json

# 8. Generate proof
snarkjs groth16 prove sudoku_final.zkey witness.wtns proof.json public.json

# 9. Verify proof
snarkjs groth16 verify verification_key.json public.json proof.json
# Must show: OK!

# 10. (Optional) Generate Solidity verifier for on-chain verification
snarkjs zkey export solidityverifier sudoku_final.zkey verifier.sol
