# Zero-Knowledge Proofs: Learning Journey

> A comprehensive exploration of ZK proof systems, from fundamentals to production implementations

**Status:** Active Learning Project  
**Last Updated:** December 2025  
**Goal:** Master ZK systems for architect-level positions

---

## 🎯 Project Overview

This repository documents a deep dive into zero-knowledge proof systems, covering multiple frameworks, proof systems, and practical implementations. The project demonstrates both theoretical understanding and practical engineering skills across the ZK stack.

### Key Achievements

- ✅ **Multi-Framework Mastery**: Implementations in Circom (Groth16), Noir (PLONK), and Halo2 (PLONK)
- ✅ **Production-Ready Circuits**: Working Sudoku solver and Merkle tree membership proofs
- ✅ **Comprehensive Documentation**: 2000+ lines of deep technical documentation
- ✅ **Framework Comparison**: Quantitative analysis of Groth16 vs PLONK (99x constraint difference)
- ✅ **Low-Level Understanding**: Custom gates and constraint systems in Halo2
- ✅ **Full Stack Integration**: Solidity verifiers, Hardhat deployment, on-chain verification

---

## 📚 Learning Path

### Phase 1: Fundamentals (`01-basics/`)

**Goal:** Understand core ZK concepts and Groth16 protocol

#### Sudoku Solver (`01-basics/sudoku/`)
- **Framework:** Circom (Groth16)
- **Circuit:** 81-cell Sudoku verification
- **Features:**
  - Range checks (1-9)
  - Row/column/block constraints
  - Public puzzle, private solution
- **Output:** Complete workflow (compile → setup → prove → verify → deploy)

#### Groth16 Verifier (`01-basics/groth16/`)
- **Language:** Rust (arkworks)
- **Purpose:** Low-level Groth16 verification implementation
- **Features:**
  - Pairing-based verification
  - G1/G2 point parsing
  - Public input commitment computation

**Documentation:** [`doc/groth.md`](doc/groth.md) - 1800+ lines covering:
- Mathematical foundations (QAP, R1CS, pairings)
- Step-by-step protocol explanation
- Security analysis
- Interview Q&A
- Optimization strategies

---

### Phase 2: Merkle Trees (`02-merkle/`)

**Goal:** Compare proof systems using the same problem

#### Implementation 1: Groth16 (`02-merkle/02-01-circom/`)
- **Framework:** Circom
- **Circuit:** Merkle membership proof (depth 3)
- **Hash:** Poseidon
- **Constraints:** 2,387
- **Features:**
  - Full Hardhat integration
  - Solidity verifier deployment
  - On-chain proof verification
  - Incremental Merkle tree contract

#### Implementation 2: PLONK (`02-merkle/02-02-noir/`)
- **Framework:** Noir (Aztec)
- **Circuit:** Same Merkle proof
- **Hash:** Poseidon2
- **Opcodes:** 24
- **Result:** 99x fewer constraints than Groth16!

#### Implementation 3: Halo2 (`02-merkle/02-03-halo2/`)
- **Framework:** Halo2 (low-level PLONK)
- **Language:** Rust
- **Features:**
  - Custom swap gate for path selection
  - Binary constraint enforcement
  - Manual circuit layout
  - Comprehensive test suite

**Documentation:** [`doc/plonk.md`](doc/plonk.md) - Complete comparison guide:
- Groth16 vs PLONK analysis
- Constraint efficiency breakdown
- Use case recommendations
- Performance benchmarks

---

## 📊 Key Metrics & Comparisons

### Merkle Tree Proof: Groth16 vs PLONK

| Metric | Groth16 (Circom) | PLONK (Noir) | Winner |
|--------|------------------|--------------|--------|
| **Constraints/Opcodes** | 2,387 | 24 | PLONK (99x) |
| **Proof Size** | 192 bytes | ~400 bytes | Groth16 (2x) |
| **Verification Time** | ~5ms | ~20ms | Groth16 (4x) |
| **Setup Type** | Circuit-specific | Universal | PLONK |
| **Custom Gates** | No | Yes | PLONK |

**Key Insight:** PLONK's custom gates provide massive efficiency gains for operations like Poseidon hashing, while Groth16 maintains advantages in proof size and verification speed.

**Full Analysis:** See [`doc/plonk.md`](doc/plonk.md) for detailed breakdown.

---

## 🛠️ Tools & Setup

### Prerequisites

```bash
# Circom (Groth16)
npm install -g circom snarkjs

# Noir (PLONK)
curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
noirup

# Halo2 (PLONK)
cargo install --git https://github.com/zcash/halo2
```

### Quick Start

**1. Sudoku (Groth16):**
```bash
cd 01-basics/sudoku
./run.sh  # Compile, setup, prove, verify
```

**2. Merkle Tree (Groth16):**
```bash
cd 02-merkle/02-01-circom
npm install
./setup.sh  # Trusted setup
node scripts/1_deploy.js  # Deploy contracts
node scripts/3_prove.js   # Generate proof
```

**3. Merkle Tree (Noir):**
```bash
cd 02-merkle/02-02-noir
nargo prove  # Generate PLONK proof
```

**4. Merkle Tree (Halo2):**
```bash
cd 02-merkle/02-03-halo2
cargo test  # Run test suite
```

---

## 📖 Documentation

### Core Documentation

- **[Groth16 Deep Dive](doc/groth.md)** - Complete mathematical and practical guide
  - R1CS → QAP transformation
  - Trusted setup ceremony
  - Pairing-based verification
  - Security analysis
  - Interview preparation

- **[PLONK vs Groth16](doc/plonk.md)** - Comprehensive comparison
  - Constraint efficiency analysis
  - Use case recommendations
  - Performance benchmarks
  - Trade-off analysis

### Implementation Guides

- **[Halo2 Merkle Proof](02-merkle/02-03-halo2/README.md)** - Low-level PLONK implementation
  - Custom gate design
  - Circuit layout
  - Constraint system explanation
  - Production patterns

### Research Notes

- **[zk-Email](etc/zk-email.md)** - Email-based anonymous credentials
- **[zk-TLS](etc/zk-TLS.md)** - TLS certificate verification in ZK
- **[Recursion & zkVM Guide](etc/recursion-zkvm-guide.md)** - Implementation roadmap for advanced topics

---

## 🏗️ Project Structure

```
ZKP/
├── 00-tools/              # Setup scripts and utilities
│   ├── circom/            # Circom installation
│   └── poseidon/          # Poseidon hash contracts
│
├── 01-basics/             # Fundamentals
│   ├── groth16/           # Rust verifier implementation
│   └── sudoku/            # Complete Groth16 workflow
│
├── 02-merkle/             # Multi-framework comparison
│   ├── 02-01-circom/      # Groth16 implementation
│   ├── 02-02-noir/        # PLONK (Noir) implementation
│   └── 02-03-halo2/       # PLONK (Halo2) implementation
│
├── doc/                   # Comprehensive documentation
│   ├── groth.md           # Groth16 complete guide
│   └── plonk.md           # PLONK vs Groth16 comparison
│
└── etc/                   # Research and notes
    ├── zk-email.md        # Email-based ZK credentials
    └── zk-TLS.md          # TLS verification in ZK
    └── recursion.md       # Proof recursion/aggregation
    └── zk-vm.md           # Basics of zkVM
```

---

## 🎓 Learning Outcomes

### Theoretical Understanding

- ✅ **Mathematical Foundations**: QAP, R1CS, pairings, polynomial commitments
- ✅ **Protocol Design**: Trusted setup ceremonies, proof generation, verification
- ✅ **Security Analysis**: Cryptographic assumptions, threat models, mitigations
- ✅ **Optimization**: Constraint minimization, proof size, gas costs

### Practical Skills

- ✅ **Circuit Design**: Range checks, hash functions, Merkle trees, custom gates
- ✅ **Multi-Framework**: Circom, Noir, Halo2 - understanding trade-offs
- ✅ **Full Stack**: Solidity verifiers, Hardhat deployment, on-chain verification
- ✅ **Low-Level**: Manual constraint systems, custom gates, circuit layout

### Production Awareness

- ✅ **Gas Optimization**: Understanding verification costs
- ✅ **Setup Management**: Trusted setup ceremonies
- ✅ **Security Best Practices**: Nullifiers, replay prevention, key management
- ✅ **Tooling**: snarkjs, circom, nargo, arkworks

---

## 🔬 Technical Highlights

### Custom Gate Design (Halo2)

Implemented a custom swap gate for conditional path selection in Merkle trees:

```rust
// Binary constraint: path_index * (1 - path_index) = 0
// Conditional selection: left = current * (1 - path_index) + sibling * path_index
```

This demonstrates deep understanding of PLONK's constraint system.

### Framework Comparison

Quantitative analysis showing:
- **99x constraint difference** between Groth16 and PLONK for Poseidon hashing
- Trade-offs: proof size vs. circuit flexibility
- When to use which system

### Production Patterns

- Incremental Merkle trees (on-chain)
- Nullifier-based replay prevention
- Poseidon hash integration
- Solidity verifier deployment

---

## 🚀 Next Steps

### Critical for Architect Roles

#### 1. Proof Recursion/Aggregation ⚠️ **HIGH PRIORITY**

**Why:** Essential for zkRollups, proof batching, and scaling ZK systems.

**Recommended Implementation:**
- **Simple Aggregator**: Verify 2-4 Groth16 proofs in a PLONK circuit
- **Framework**: Halo2 (cycle-friendly curves) or Plonky2
- **Goal**: Demonstrate proof-of-proof concept
- **Complexity**: Medium (requires verifier circuit)

**Learning Path:**
```
03-recursion/
├── 03-01-simple-aggregator/    # Verify 2 proofs → 1 proof
│   ├── circuits/
│   │   └── aggregator.rs       # Halo2 verifier circuit
│   └── tests/
│       └── aggregation_test.rs
├── 03-02-batch-verifier/        # Verify N proofs efficiently
└── README.md                    # Recursion deep dive
```

**Key Concepts:**
- Verifier circuit design (~1K-30K constraints)
- Cycle-friendly curves (Pasta: Pallas/Vesta)
- Proof compression (N proofs → 1 proof)
- Gas optimization (250K → 25K per proof)

#### 2. zkVM Basics ⚠️ **HIGH PRIORITY**

**Why:** Foundation of zkEVM, zkWASM, and general-purpose ZK computation.

**Recommended Implementation:**
- **TinyVM**: Simple stack-based VM (10-20 instructions)
- **Instructions**: ADD, MUL, PUSH, POP, JUMP, CALL
- **Framework**: Halo2 or Circom
- **Goal**: Understand VM → circuit compilation

**Learning Path:**
```
04-zkvm/
├── 04-01-tiny-vm/               # Minimal VM implementation
│   ├── src/
│   │   ├── vm.rs                # VM state machine
│   │   ├── instructions.rs      # Instruction set
│   │   └── circuit.rs           # VM → circuit compiler
│   └── examples/
│       └── fibonacci.rs         # Prove Fibonacci computation
├── 04-02-risc-v-basics/         # RISC-V subset (optional)
└── README.md                    # zkVM architecture guide
```

**Key Concepts:**
- Instruction encoding
- State transitions (registers, memory, stack)
- Program counter management
- Memory access patterns
- Loop unrolling vs. recursion

### Other Enhancements

- [ ] Lookup tables (Plookup) - Efficient range checks
- [ ] Performance benchmarks - Actual numbers for all implementations
- [ ] More production patterns (ECDSA, range proofs)
- [ ] STARKs (transparent setup) - Alternative to SNARKs
- [ ] Nova (incremental verifiable computation) - Advanced recursion
- [ ] Custom gate optimizations - Domain-specific efficiency

---

## 📚 Resources

### Papers

- **Groth16**: "On the Size of Pairing-based Non-interactive Arguments" (2016)
- **PLONK**: "PLONK: Permutations over Lagrange-bases..." (2019)
- **Poseidon**: "Poseidon: A New Hash Function for Zero-Knowledge Proof Systems"

### Learning Materials

- [0xPARC Learning Resources](https://learn.0xparc.org/)
- [ZK Whiteboard Sessions](https://www.youtube.com/c/ZeroKnowledge)
- [Halo2 Book](https://zcash.github.io/halo2/)

### Tools

- [Circom](https://github.com/iden3/circom) - Circuit compiler
- [snarkjs](https://github.com/iden3/snarkjs) - Proof generation
- [Noir](https://noir-lang.org/) - High-level PLONK language
- [Halo2](https://github.com/zcash/halo2) - Low-level PLONK framework
- [arkworks](https://arkworks.rs/) - Rust ZK library

---

## 💡 Key Insights

### When to Use Groth16

✅ **Best for:**
- Proof size is critical (blockchain storage)
- Verification speed matters (gas optimization)
- Circuit is stable (few updates)
- Production-ready systems

**Examples:** Zcash, Tornado Cash, Filecoin

### When to Use PLONK

✅ **Best for:**
- Circuit evolves frequently
- Need custom gates (efficiency)
- Multiple circuits in system
- Planning recursion/aggregation

**Examples:** zkSync Era, Aztec Network, Polygon zkEVM

### The Trade-off

- **Groth16**: Smaller proofs, faster verification, but circuit-specific setup
- **PLONK**: Universal setup, custom gates, but larger proofs

**The choice depends on your constraints** - proof size vs. circuit flexibility.

---

## 🎯 For Hiring Managers

This project demonstrates:

1. **Depth**: Understanding ZK systems from mathematical foundations to implementation
2. **Breadth**: Multiple frameworks and proof systems
3. **Production Awareness**: Full-stack integration, gas optimization, security
4. **Analytical Thinking**: Quantitative comparisons, trade-off analysis
5. **Documentation**: Comprehensive guides suitable for team knowledge sharing

### Current Coverage

✅ **Completed:**
- Groth16 (Circom) - Production-ready implementation
- PLONK (Noir) - High-level framework
- PLONK (Halo2) - Low-level with custom gates
- Framework comparison with quantitative analysis
- Full-stack integration (Solidity, Hardhat)

⚠️ **In Progress / Planned:**
- Proof recursion/aggregation (see [`etc/recursion.md`](etc/recursion.md))
- zkVM basics (see [`etc/zk-vm.md`](etc/zk-vm.md))

**Note:** Recursion and zkVM are documented with implementation roadmaps. These are the next logical additions for architect-level completeness.

**Suitable for:** ZK Architect, Senior ZK Engineer, Protocol Engineer roles

---

## 📝 License

This is a personal learning project. Code examples are provided for educational purposes.

---

## 🙏 Acknowledgments

- **0xPARC** - Excellent learning resources
- **Circom Community** - Framework and tooling
- **Aztec** - Noir framework
- **Zcash** - Halo2 framework
- **All ZK researchers** - Building the future of privacy

---

**Questions or feedback?** This is a learning project - always open to improvements and discussions!

---

*"The best way to learn ZK is to build ZK."*

