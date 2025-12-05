# Zero-Knowledge Proofs Architect Roadmap

**3-6 Month Learning Plan**  
**From Fundamentals to Production**

*Created: December 2025*

---

## Table of Contents

1. [Overview & Timeline](#1-overview--timeline)
2. [Phase 1: Mathematical Foundations](#2-phase-1-mathematical-foundations)
3. [Phase 2: Practical ZK Systems](#3-phase-2-practical-zk-systems)
4. [Phase 3: Production Best Practices](#4-phase-3-production-best-practices)
5. [Phase 4: Interview Preparation](#5-phase-4-interview-preparation)
6. [Resources & References](#6-resources--references)
7. [Project Templates](#7-project-templates--starter-code)

---

## 1. Overview & Timeline

This roadmap provides a structured path from ZK fundamentals to becoming job-ready for ZKP Architect roles at companies like zkSync, Aztec, StarkWare, and Scroll. The plan balances mathematical understanding with practical implementation experience.

### Timeline Overview

| Phase | Duration | Focus | Deliverable |
|-------|----------|-------|-------------|
| Phase 1 | 3-4 weeks | Math Foundations | Understanding core concepts |
| Phase 2 | 6-8 weeks | Practical Projects | 4 portfolio projects |
| Phase 3 | 2-3 weeks | Production | Protocol analysis |
| Phase 4 | 2-3 weeks | Interview Prep | Job applications |

### Success Criteria

- ✅ Can explain Groth16, PLONK, and STARKs trade-offs
- ✅ 4+ production-quality ZK projects on GitHub
- ✅ Understand common security vulnerabilities in circuits
- ✅ Experience optimizing circuits for performance
- ✅ Familiar with at least one production ZK protocol codebase

---

## 2. Phase 1: Mathematical Foundations

**Duration: 3-4 weeks**

> **Goal:** Understand the mathematics underlying ZK proofs sufficiently to read research papers and explain verification algorithms. You don't need a PhD - focus on intuition and practical understanding.

### Week 1: Finite Fields & Elliptic Curves

| Topic | What to Learn | Why It Matters |
|-------|---------------|----------------|
| **Finite Fields** | • Modular arithmetic<br>• Field operations<br>• Multiplicative groups | All ZK arithmetic happens in finite fields |
| **Elliptic Curves** | • Point addition<br>• Scalar multiplication<br>• Group law | Foundation of all pairing-based ZK systems |
| **Practical** | Implement scalar multiplication in Rust | Understand computational cost |

**Resources:**
- **Moonmath Manual** - Chapters 1-3 (free PDF, very thorough)
- **Andrea Corbellini's ECC Tutorial** - Visual explanations
- **Practice:** Implement curve operations using ark-ec primitives

> **Checkpoint:** Can you explain why y² = x³ + ax + b defines a group?

### Week 2: Bilinear Pairings

Pairings are the "magic" that makes Groth16 and PLONK work. A pairing e(P, Q) is a function that maps two elliptic curve points to a field element with special properties.

**Key Properties:**
- **Bilinearity:** e(aP, bQ) = e(P, Q)^(ab) - allows verification without knowing a or b
- **Non-degeneracy:** If e(P, Q) = 1 for all Q, then P = 0
- **Computability:** Can be efficiently computed (but expensive)

> **Key Insight:** Pairings let us verify polynomial relationships without revealing the polynomial!

**Resources:**
- **Vitalik's 'Exploring Elliptic Curve Pairings'** - Best intuitive explanation
- **Ben Lynn's Thesis** - Chapter 2 (more technical but thorough)
- **Practice:** Trace through your Groth16 verifier line-by-line understanding each pairing

> **Checkpoint:** Explain why e(A, B) = e(α, β) · e(inputs, γ) · e(C, δ) proves correctness

### Weeks 3-4: Polynomial Commitments & KZG

> **The Core Idea:** Modern ZK systems encode computation as polynomials, then prove properties about those polynomials. KZG commitments let you commit to a polynomial and later prove evaluations without revealing the polynomial.

| Concept | Explanation |
|---------|-------------|
| **Commitment** | Commit to polynomial p(x) as C = [p(τ)]₁ where τ is secret |
| **Evaluation Proof** | Prove p(z) = y without revealing p(x) |
| **Trusted Setup** | Generate powers of τ: [τ⁰]₁, [τ¹]₁, ..., [τⁿ]₁ |
| **Verification** | Use pairings to check evaluation correctness |

**Practical Exercise: Implement Toy KZG**

1. Choose small prime p and polynomial p(x) = x² + 3x + 2
2. Generate trusted setup: random τ, compute [τ⁰], [τ¹], [τ²]
3. Create commitment: C = p(τ)·G
4. Prove evaluation: p(5) = 42
5. Verify proof using pairing equation

**Resources:**
- **Dankrad Feist's KZG explainer** - Excellent visual walkthrough
- **Justin Drake's videos** - Youtube channel has great ZK content
- **arkworks-rs examples** - See poly-commit examples

### Phase 1 Success Criteria

| ✓ | **Phase 1 Success Criteria** |
|---|------------------------------|
| □ | Understand finite field arithmetic and can compute examples |
| □ | Can explain elliptic curve group law |
| □ | Understand what pairings do (even if not how they're computed) |
| □ | Can explain KZG commitment scheme |
| □ | Successfully traced through Groth16 verification equation |

---

## 3. Phase 2: Practical ZK Systems

**Duration: 6-8 weeks**

> **Goal:** Build portfolio projects demonstrating hands-on ZK expertise. Each project should be production-quality with good documentation - these will be your interview talking points.

### Project 1: Private Voting System

**Duration: 2 weeks | Difficulty: ★★☆☆☆**

Build a voting system where users prove they voted validly (0 or 1) without revealing their choice. This is a classic ZK application and interview favorite.

#### Week 1: Circuit Design

- Design circuit constraints: vote ∈ {0, 1}, user has voting credential
- Implement in Circom or Noir (recommend Circom for learning)
- Test with various inputs including edge cases
- Optimize constraint count - aim for < 100 constraints

#### Week 2: Smart Contract Integration

- Generate Solidity verifier contract
- Write voting contract that verifies proofs on-chain
- Add vote tallying logic (ZK proofs prove individual votes valid)
- Deploy to testnet (Sepolia or similar)
- Write comprehensive README with demo

#### Deliverables

| Deliverable | Checklist |
|-------------|-----------|
| **Circuit code** | ✓ Well-commented Circom/Noir code<br>✓ Test suite with 10+ test cases |
| **Smart contract** | ✓ Solidity verifier contract<br>✓ Voting logic contract<br>✓ Deployed to testnet |
| **Documentation** | ✓ README with architecture<br>✓ Demo video/screenshots<br>✓ Gas cost analysis |

**Pro Tips:**
- Use Merkle trees for voter registry (more gas efficient)
- Consider double-voting prevention strategies
- Document trade-offs: privacy vs auditability
- Benchmark: proof generation time, verification gas cost

---

### Project 2: Computation Verification (Groth16 vs PLONK)

**Duration: 2 weeks | Difficulty: ★★★☆☆**

Compare two major ZK systems by implementing the same computation in both. This demonstrates understanding of trade-offs - crucial for architect interviews.

**The Challenge:** Prove that Fibonacci(n) = X without revealing intermediate values. Implement in both Groth16 (Circom + snarkjs) and PLONK (Halo2 or Noir).

#### Week 1: Groth16 Implementation

- Write Fibonacci circuit in Circom (iterative, not recursive)
- Optimize constraints - can you get under 2n constraints?
- Measure proof generation for n=100, 1000, 10000
- Document trusted setup ceremony requirements

#### Week 2: PLONK Implementation & Comparison

- Implement same circuit in Halo2 or Noir
- Measure: proof size, gen time, verification time
- Create comparison table for different n values
- Write analysis: when to use which system

#### Comparison Metrics

| Metric | Groth16 | PLONK | Winner |
|--------|---------|-------|--------|
| Proof Size | ~200 bytes | ~50KB | Groth16 |
| Trusted Setup | Circuit-specific | Universal | PLONK |
| Prover Time | Moderate | Slower | Groth16 |
| Verifier Time | Fast (~2ms) | Moderate | Groth16 |

**Key Takeaways to Document:**
- Groth16: Best for repeated proofs of same circuit (rollups)
- PLONK: Better for varied circuits, universal setup
- Trade-off: proof size vs setup flexibility
- Neither is 'better' - depends on use case

---

### Project 3: Merkle Tree Membership Proof

**Duration: 2 weeks | Difficulty: ★★★★☆**

Prove you own an element in a Merkle tree without revealing which one. This is used in privacy protocols (Tornado Cash), zkRollups, and private credentials. Critical production pattern.

**Technical Requirements:**
- Tree depth: 20 levels (1 million leaves)
- Hash function: Poseidon (ZK-friendly)
- Private inputs: leaf value, Merkle path
- Public inputs: root hash
- Constraint budget: < 20,000

#### Week 1: Circuit Optimization

- Implement naive version first (baseline constraints)
- Optimize hash operations - batch where possible
- Consider alternative tree structures (quaternary?)
- Profile constraint usage per operation
- Goal: Reduce constraints by 30%+ from naive

#### Week 2: Integration & Performance

- Build off-chain Merkle tree manager (Rust)
- Generate proofs for random paths
- Benchmark: proof gen time across tree sizes
- Create visualization of tree + proof path
- Document memory usage and optimization techniques

**Optimization Techniques to Explore:**
- Use Poseidon instead of SHA256 (10x fewer constraints)
- Batch sibling node processing
- Precompute as much as possible outside circuit
- Consider trade-off: tree depth vs constraint count

> **Interview Gold:** This project demonstrates constraint optimization skills - highly valued!

---

### Project 4: Recursive Proof Composition

**Duration: 2-3 weeks | Difficulty: ★★★★★**

> **Advanced Topic:** Verify a ZK proof inside another ZK proof. This enables proof aggregation (rollups), incrementally verifiable computation (IVC), and is cutting-edge - few engineers understand this.

**Why This Matters:**
- **Rollups:** Aggregate 1000s of transaction proofs into one
- **Scaling:** Constant verification cost regardless of computation
- **IVC:** Long-running computations split into steps
- **Job Market:** Very few people know this - instant differentiation

#### Recommended Approach

| Phase | Tasks | Complexity |
|-------|-------|------------|
| **Setup** | • Choose system: Halo2 or Nova<br>• Understand cycle of curves<br>• Read Halo2 book thoroughly | High |
| **Week 1-2** | • Implement base circuit (e.g., hash chain)<br>• Verify one proof inside another<br>• Debug (this is hard!) | Very High |
| **Week 3** | • Chain multiple proofs<br>• Benchmark aggregation savings<br>• Document architecture | High |

**Resources:**
- **Halo2 Book** - Official documentation, very detailed
- **Nova paper** - Folding schemes (simpler than full recursion)
- **Zcash blog** - Halo 2 announcement and explainer
- **PSE (Privacy & Scaling Explorations) examples** - Production code

> **Reality Check:** This is genuinely hard. It's okay if you don't fully complete it - the attempt shows ambition. Document what you learned!

### Phase 2 Portfolio Checklist

| ✓ | **Phase 2 Portfolio Checklist** |
|---|----------------------------------|
| □ | Project 1: Private voting (GitHub + demo) |
| □ | Project 2: Groth16 vs PLONK comparison (with data) |
| □ | Project 3: Optimized Merkle proof (< 20K constraints) |
| □ | Project 4: Recursive proof attempt (document learnings) |
| □ | All projects have READMEs with architecture diagrams |
| □ | Each project has < 5 min demo video or screenshots |

---

## 4. Phase 3: Production Best Practices

**Duration: 2-3 weeks**

> **Goal:** Understand how ZK is used in production systems, common pitfalls, and optimization techniques. This separates 'did tutorials' from 'can work in production.'

### Task 1: Production Protocol Analysis

**Duration: 1-2 weeks**

Choose ONE protocol to study deeply. Quality over quantity - better to truly understand one system than superficially know three.

| Protocol | Why Choose | Key Learnings |
|----------|------------|---------------|
| **zkSync Era** | • EVM compatible<br>• Large codebase<br>• Active development | • zkEVM challenges<br>• Prover architecture<br>• Batch processing |
| **Scroll** | • Also zkEVM<br>• Different architecture<br>• Good docs | • Circuit modularity<br>• Prover coordination<br>• Gas optimization |
| **StarkNet** | • Uses STARKs (no trusted setup)<br>• Cairo language<br>• Different paradigm | • STARK vs SNARK<br>• Cairo VM design<br>• Transparent proofs |
| **Aztec** | • Privacy-focused<br>• Novel architecture<br>• Noir language | • Private state<br>• Encrypted mempools<br>• Recursive proofs |

**Analysis Framework:**
- **Architecture:** Draw system diagram - prover, verifier, sequencer, coordinator
- **Circuit Design:** How are circuits structured? Modular? Monolithic?
- **Prover Setup:** Hardware requirements, parallelization, optimizations
- **Bottlenecks:** What's the slowest part? Where would you optimize?
- **Trade-offs:** What design choices were made and why?

> **Deliverable:** 5-10 page technical analysis document with diagrams

---

### Task 2: Common Pitfalls & Security

**Duration: 3-5 days**

Study real bugs and vulnerabilities in ZK systems. Understanding what goes wrong is as important as knowing what goes right.

**Bug Categories to Study:**

| Category | Example | Impact |
|----------|---------|--------|
| **Under-constrained** | Missing range check allows invalid witness | Critical - invalid proofs accepted |
| **Trusted setup** | Toxic waste not destroyed properly | Critical - proof system compromised |
| **Arithmetic errors** | Overflow in finite field operations | High - incorrect results |
| **Implementation bugs** | Prover generates but verifier rejects | Medium - DoS but no security breach |

**Resources:**
- **Trail of Bits ZK Bug Tracker** - Comprehensive list of real bugs
- **Audit reports:** Read OpenZeppelin, Trail of Bits, Zellic audits of ZK projects
- **0xPARC blog** - 'Common circuit bugs' series

> **Exercise:** Find a bug in one of your own circuits. Then fix it and document the lesson.

---

### Task 3: Circuit Optimization

**Duration: 3-5 days**

Take your Sudoku circuit from Phase 0 and optimize it. This hands-on exercise will teach you practical optimization techniques.

**Optimization Strategies:**

| Strategy | Technique | Expected Savings |
|----------|-----------|------------------|
| **Hash function** | Switch SHA256 → Poseidon | 90% constraint reduction |
| **Range checks** | Use lookup tables instead of constraints | 50-70% reduction |
| **Packing** | Pack multiple bits into field elements | 30-50% reduction |
| **Precomputation** | Move computations outside circuit when possible | 20-40% reduction |

**Optimization Exercise Steps:**

1. Measure baseline: constraint count, proof time, verification time
2. Profile: which constraints are most expensive?
3. Apply optimization techniques one at a time
4. Measure impact of each change
5. Document trade-offs (e.g., complexity vs performance)

### Phase 3 Success Criteria

| ✓ | **Phase 3 Success Criteria** |
|---|-------------------------------|
| □ | Completed analysis of one production protocol |
| □ | Can identify and explain 3+ common circuit vulnerabilities |
| □ | Optimized a circuit by at least 30% |
| □ | Documented optimization techniques and trade-offs |
| □ | Understand prover/verifier architecture in production |

---

## 5. Phase 4: Interview Preparation

**Duration: 2-3 weeks**

> **Goal:** Convert your technical knowledge into job offers. ZKP architect interviews are technically deep but follow predictable patterns.

### Typical Interview Process

| Round | Format | What They're Testing |
|-------|--------|----------------------|
| **1. Recruiter** | 30 min call | • Communication skills<br>• Genuine interest<br>• Timeline/compensation |
| **2. Technical Screen** | 60 min<br>Phone/Zoom | • ZK fundamentals<br>• Can explain concepts clearly<br>• Problem-solving approach |
| **3. Coding Round** | 90 min<br>Live coding | • Circuit design<br>• Debug constraints<br>• Optimization thinking |
| **4. System Design** | 60 min<br>Whiteboard | • Architecture skills<br>• Trade-off analysis<br>• Production thinking |
| **5. Team/Culture** | 30-45 min | • Team fit<br>• Collaboration<br>• Values alignment |

### Common Interview Questions

#### Theoretical Questions

- Explain Groth16 vs PLONK vs STARKs - when would you use each?
- What is a trusted setup and why does it matter? How do you make it secure?
- How do pairings enable succinct proofs? Walk me through the verification equation.
- What are polynomial commitments and why are they important?
- Explain recursive proof composition. What are the challenges?
- How does zkEVM work? What are the main technical challenges?

#### Practical Questions

- Design a circuit to prove X (e.g., age > 18 without revealing age)
- This circuit has a bug - find it (under-constrained circuit)
- How would you optimize this circuit? What's the constraint count?
- Walk me through one of your projects - what were the challenges?
- How would you debug a proof that generates but doesn't verify?

#### System Design Questions

- Design a zkRollup from scratch - what are the components?
- How would you make proof generation 10x faster? What are the bottlenecks?
- Design a privacy-preserving DeFi protocol using ZK proofs
- You have 1000s of users generating proofs - how do you architect the prover?
- Trade-offs between different proof systems for our use case?

---

### Portfolio Walk-Through Preparation

You'll be asked to walk through your projects. Prepare a 5-minute presentation for each project following this structure:

| Section | What to Cover | Time |
|---------|---------------|------|
| **Problem** | • What does this solve?<br>• Why is it interesting? | 30 sec |
| **Approach** | • High-level architecture<br>• Key technical decisions | 1 min |
| **Challenges** | • What was hard?<br>• How did you solve it? | 1.5 min |
| **Results** | • Performance metrics<br>• Trade-offs made | 1 min |
| **Learnings** | • What would you do differently?<br>• What's next? | 1 min |

> **Pro Tip:** Practice explaining to someone non-technical. If you can explain it simply, you truly understand it.

---

### Application Strategy

- **Timing:** Start applying end of Month 2 (with 2-3 projects done)
- **Quantity:** Apply to 10-15 companies, expect 3-5 responses
- **Network:** Engage on Twitter/X, contribute to open source, attend ZK events
- **Custom applications:** Reference their specific tech in your cover letter
- **Follow up:** If no response in 2 weeks, polite follow-up email

#### Target Companies (Priority Order)

| Tier | Companies | Notes |
|------|-----------|-------|
| **Tier 1<br>(Hardest)** | • Aztec<br>• Scroll<br>• zkSync<br>• StarkWare<br>• Succinct<br>• Risc Zero | Top comp, most competitive.<br>Apply to 2-3. |
| **Tier 2<br>(Strong)** | • Polygon Zero/Miden<br>• Hyperlane<br>• Polyhedra<br>• O(1) Labs | Great projects, solid comp.<br>Apply to 3-4. |
| **Tier 3<br>(Emerging)** | • Various L1/L2s adding ZK<br>• ZK infrastructure startups<br>• Research labs | Good entry points.<br>Apply to 4-5. |

### Phase 4 Preparation Checklist

| ✓ | **Phase 4 Preparation Checklist** |
|---|------------------------------------|
| □ | Can explain all major ZK systems (Groth16, PLONK, STARKs) |
| □ | Practiced 5-min presentations for each portfolio project |
| □ | Prepared answers to top 10 common questions |
| □ | Mock interview practice (at least 2 sessions) |
| □ | GitHub profile polished and professional |
| □ | Applications sent to 10+ companies |
| □ | Active on ZK Twitter/X (or relevant community) |

---

## 6. Resources & References

### Essential Reading

| Resource | Type | Why Read It |
|----------|------|-------------|
| **Moonmath Manual** | Book (Free PDF) | Most comprehensive ZK math reference |
| **Proofs, Arguments, and ZK<br>(Thaler)** | Book | Excellent theoretical foundation |
| **Why and How zk-SNARKs Work<br>(Petkus)** | Article | Best intuitive Groth16 explainer |
| **Halo 2 Book** | Documentation | Learn recursive proofs |
| **PLONK Paper<br>(Gabizon et al.)** | Paper | Understand universal setup |

### Key Online Resources

- **0xPARC Learning Group** - https://learn.0xparc.org/ - Excellent tutorials
- **ZK Whiteboard Sessions** - Youtube series by Dan Boneh - Video lectures
- **ZK Hack** - https://zkhack.dev/ - Puzzles and challenges
- **Circom Documentation** - https://docs.circom.io/ - Circuit language
- **arkworks** - https://github.com/arkworks-rs - Rust ZK libraries
- **PSE (Ethereum Foundation)** - https://github.com/privacy-scaling-explorations - Production code examples

### Community & Networking

| Platform | Key Accounts/Channels |
|----------|----------------------|
| **Twitter/X** | • @VitalikButerin<br>• @dankrad<br>• @aztecnetwork<br>• @0xPolygon<br>• @zksync<br>• @StarkWareLtd |
| **Discord** | • PSE Discord<br>• zkSync Discord<br>• Scroll Discord<br>• 0xPARC |
| **Telegram** | • ZK Research<br>• Various protocol channels |

### Essential Tools & Libraries

**Circuit Languages:**
- Circom - Most popular, good for learning
- Noir (Aztec) - Rust-like syntax, growing ecosystem
- Cairo (StarkNet) - For STARKs
- Halo2 - Rust DSL, advanced features

**Rust Libraries:**
- arkworks - Core cryptographic primitives
- bellman - Groth16 implementation
- halo2 - Recursive proof system

**JavaScript:**
- snarkjs - Groth16 tooling
- circomlib - Circuit template library

---

## 7. Project Templates & Starter Code

Use these templates as starting points for your projects. Each includes boilerplate code, project structure, and documentation templates.

### Project README Template

Every project should have a README following this structure:

```markdown
# Project Name

## Overview
[1-2 paragraph description of what this project does and why it's interesting]

## Technical Details
- **Proof System:** Groth16 / PLONK / STARKs
- **Circuit Language:** Circom / Noir / Halo2
- **Constraints:** [number]
- **Proof Size:** [bytes]
- **Generation Time:** [ms/s]
- **Verification Time:** [ms]

## Architecture
[Include a diagram showing the flow: input → circuit → proof → verifier]

## Installation
```bash
# Installation steps
```

## Usage
```bash
# How to run
```

## Results
[Performance data, optimization results, benchmarks]

## Challenges & Learnings
[What was hard? What did you learn? What would you do differently?]

## Future Work
[What could be improved or extended?]

## References
[Links to papers, resources used]

### Recommended Project Structure

Use this directory structure for consistency:

```
project-name/
├── README.md
├── circuits/
│   ├── main.circom (or .noir, .rs)
│   ├── helpers.circom
│   └── tests/
├── contracts/
│   ├── Verifier.sol (generated)
│   └── YourContract.sol
├── scripts/
│   ├── compile.sh
│   ├── setup.sh
│   ├── prove.sh
│   └── verify.sh
├── test/
│   └── integration_tests.js
├── docs/
│   ├── architecture.md
│   ├── optimization.md
│   └── diagrams/
└── benchmarks/
    └── results.csv
```

---

## Final Notes

This roadmap is ambitious but achievable. The key is consistency - even 1-2 hours per day adds up. Don't aim for perfection; aim for progress. Document your journey, share your learnings, and engage with the community.

ZKP is a fast-moving field. By the time you complete this roadmap, new systems may have emerged. That's okay - the fundamentals you'll learn (elliptic curves, pairings, polynomial commitments) remain constant. You'll have the foundation to quickly pick up new systems.

Remember: The goal isn't just to get a job - it's to become a genuinely skilled ZKP engineer who can contribute meaningfully to this technology.

**Good luck! 🚀**

---

### How to Use This Document

- Print sections as you work through them
- Check off boxes as you complete tasks
- Copy sections back to Claude for detailed guidance on each topic
- Update with your own notes and insights

