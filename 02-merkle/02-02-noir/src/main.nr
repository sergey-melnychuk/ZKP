use dep::poseidon::poseidon2::Poseidon2;

fn main(
    // Private inputs
    secret: Field,
    siblings: [Field; 3],
    path_indices: [u1; 3],
    
    // Public inputs
    root: pub Field,
    nullifier: pub Field
) {
    // 1. Compute leaf from secret using Poseidon2
    let leaf = Poseidon2::hash([secret], 1);
    // let leaf = std::hash::pedersen_hash([secret]);
    
    // 2. Verify nullifier
    let computed_nullifier = Poseidon2::hash([secret], 1);
    // let computed_nullifier = std::hash::pedersen_hash([secret]);

    assert(nullifier == computed_nullifier);
    // std::println(f"Computed nullifier: {computed_nullifier}");
    // std::println(f"Input nullifier: {nullifier}");

    // 3. Climb Merkle tree
    let mut current_hash = leaf;
    
    for i in 0..3 {
        let sibling = siblings[i];
        let is_left = path_indices[i];
        
        // Compute left and right based on path
        let (left, right) = if is_left == 0 {
            (current_hash, sibling)
        } else {
            (sibling, current_hash)
        };
        
        current_hash = Poseidon2::hash([left, right], 2);
        // current_hash = std::hash::pedersen_hash([left, right]);
    }
    
    // 4. Verify root
    assert(current_hash == root);

    // std::println(f"Computed root: {current_hash}");
    // std::println(f"Input root: {root}");
}
