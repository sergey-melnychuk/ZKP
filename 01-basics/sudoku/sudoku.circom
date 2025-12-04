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

component main {public [puzzle]} = Sudoku();
