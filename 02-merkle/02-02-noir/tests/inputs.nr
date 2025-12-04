use dep::poseidon::poseidon2::Poseidon2;

#[test]
fn compute_merkle_inputs() {
    let secret = 12345;
    
    // Compute leaf and nullifier
    let leaf = Poseidon2::hash([secret], 1);
    let nullifier = Poseidon2::hash([secret], 1);
    
    // Compute root for empty tree (all siblings are 0)
    let mut current = leaf;
    for _i in 0..3 {
        current = Poseidon2::hash([current, 0], 2);
    }
    let root = current;
    
    // Print values
    println(f"secret = \"{secret}\"");
    println(f"siblings = [\"0\", \"0\", \"0\"]");
    println(f"path_indices = [0, 0, 0]");
    println(f"root = \"{root}\"");
    println(f"nullifier = \"{nullifier}\"");
    
    assert(true);
}

#[test]
fn compute_inputs() {
    let secret = 12345;
    
    let leaf = Poseidon2::hash([secret], 1);
    let nullifier = Poseidon2::hash([secret], 1);
    
    let mut current = leaf;
    for _i in 0..3 {
        current = Poseidon2::hash([current, 0], 2);
    }
    let root = current;
    
    std::println(f"secret = \"{secret}\"");
    std::println(f"siblings = [\"0\", \"0\", \"0\"]");
    std::println(f"path_indices = [0, 0, 0]");
    std::println(f"root = \"{root}\"");
    std::println(f"nullifier = \"{nullifier}\"");
}

