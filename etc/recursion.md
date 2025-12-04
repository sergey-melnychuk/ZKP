## Proof Recursion

### What is Recursion?

**Goal:** Verify a proof inside another proof.

**Use Case:** Instead of verifying 1000 proofs separately (1000 × 250K gas = 250M gas), verify them all in one recursive proof (~280K gas total).

### Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│              Proof Recursion Architecture                │
└─────────────────────────────────────────────────────────┘

Base Layer (Groth16):
  Proof 1: Merkle proof (192 bytes)
  Proof 2: Merkle proof (192 bytes)
  Proof 3: Merkle proof (192 bytes)
  ...
  Proof N: Merkle proof (192 bytes)
            ↓
┌─────────────────────────────────────────────────────────┐
│  Aggregation Layer (PLONK/Halo2)                        │
│                                                          │
│  Verifier Circuit:                                      │
│  - Verify Proof 1 (30K constraints)                   │
│  - Verify Proof 2 (30K constraints)                    │
│  - Verify Proof 3 (30K constraints)                    │
│  - ...                                                   │
│  - Aggregate public inputs                              │
│  - Output: Single aggregated proof                      │
│                                                          │
│  Result: 1 proof instead of N proofs                    │
└─────────────────────────────────────────────────────────┘
            ↓
  Final Proof: ~400 bytes (PLONK)
  Verification: ~280K gas (vs 250M gas for N proofs)
```

### Implementation Plan

#### Phase 1: Simple Aggregator (2 proofs → 1 proof)

**Framework:** Halo2 (cycle-friendly curves)

**Goal:** Verify 2 Groth16 proofs in a PLONK circuit

**Steps:**

1. **Create Verifier Circuit**
   ```rust
   // Verify Groth16 proof in Halo2
   struct Groth16Verifier {
       proof_a: G1Point,
       proof_b: G2Point,
       proof_c: G1Point,
       public_inputs: Vec<Fr>,
       vk: VerificationKey,
   }
   
   // Constraint: e(A, B) = e(α, β) · e(vk_x, γ) · e(C, δ)
   // This is ~1K-30K constraints depending on optimization
   ```

2. **Aggregate Public Inputs**
   ```rust
   // Combine public inputs from both proofs
   struct AggregatedInputs {
       root1: Fr,
       root2: Fr,
       // Aggregate into single commitment
       aggregated_root: Fr,
   }
   ```

3. **Generate Recursive Proof**
   ```rust
   // Prove: "I verified 2 Groth16 proofs correctly"
   let aggregated_proof = prove_aggregation(
       proof1, proof2, public_inputs1, public_inputs2
   );
   ```

**Expected Results:**
- Verifier circuit: ~2K-60K constraints (2 × 30K)
- Proof size: ~400 bytes (PLONK)
- Verification gas: ~300K (vs 500K for 2 separate proofs)

#### Phase 2: Batch Aggregator (N proofs → 1 proof)

**Goal:** Scale to arbitrary N

**Challenges:**
- Circuit size grows with N
- Need efficient batching strategy
- Memory management for large N

**Solutions:**
- Tree-based aggregation (log N depth)
- Fixed-size batches (e.g., 10 proofs per batch)
- Use Nova for better scaling

#### Phase 3: Production Patterns

**Add:**
- Gas optimization techniques
- Batch size optimization
- Parallel proof generation
- On-chain aggregation contract

### Recommended Structure

```
03-recursion/
├── 03-01-simple-aggregator/
│   ├── src/
│   │   ├── verifier_circuit.rs      # Groth16 verifier in Halo2
│   │   ├── aggregator.rs            # 2-proof aggregator
│   │   └── lib.rs
│   ├── tests/
│   │   └── aggregation_test.rs
│   └── README.md                    # Implementation guide
│
├── 03-02-batch-aggregator/
│   ├── src/
│   │   ├── batch_verifier.rs        # N-proof aggregator
│   │   └── tree_aggregation.rs     # Tree-based approach
│   └── benchmarks/
│       └── gas_analysis.md
│
├── 03-03-nova/                      # Advanced: Nova IVC
│   └── README.md                    # Nova deep dive
│
└── README.md                        # Recursion overview
```

### Learning Resources

**Papers:**
- "Nova: Recursive SNARKs without trusted setup" (2022)
- "Cycle of Curves" - Pasta curves for recursion
- "Proof-Carrying Data" - Original recursion concept

**Code:**
- [Halo2 Examples](https://github.com/zcash/halo2/tree/main/halo2_proofs/examples) - Has recursion examples
- [Plonky2](https://github.com/mir-protocol/plonky2) - Recursive SNARK framework
- [Nova](https://github.com/microsoft/Nova) - IVC implementation

**Tutorials:**
- [0xPARC Recursion Guide](https://learn.0xparc.org/)
- Halo2 Book - Recursion chapter
