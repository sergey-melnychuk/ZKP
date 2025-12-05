## zkVM Basics

### What is a zkVM?

**Goal:** Prove arbitrary computation by compiling a VM into a circuit.

**Use Case:** Instead of writing circuits for each program, write a VM circuit once, then prove any program runs correctly on that VM.

### Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    zkVM Architecture                    │
└─────────────────────────────────────────────────────────┘

Program (High-level):
  function fibonacci(n) {
      if n <= 1: return n
      return fibonacci(n-1) + fibonacci(n-2)
  }
            ↓
┌─────────────────────────────────────────────────────────┐
│  Compilation: Program → Bytecode                        │
│                                                         │
│  Bytecode:                                              │
│    PUSH 1                                               │
│    PUSH n                                               │
│    LT                                                   │
│    JUMP_IF_FALSE label1                                 │
│    PUSH n                                               │
│    RETURN                                               │
│  label1:                                                │
│    CALL fibonacci(n-1)                                  │
│    CALL fibonacci(n-2)                                  │
│    ADD                                                  │
│    RETURN                                               │
└─────────────────────────────────────────────────────────┘
            ↓
┌─────────────────────────────────────────────────────────┐
│  VM Circuit (Halo2/Circom)                              │
│                                                         │
│  State:                                                 │
│    - Program counter (PC)                               │
│    - Stack (values)                                     │
│    - Memory (optional)                                  │
│    - Registers (optional)                               │
│                                                         │
│  Instructions:                                          │
│    - PUSH: Add value to stack                           │
│    - POP: Remove from stack                             │
│    - ADD: Pop 2, push sum                               │
│    - MUL: Pop 2, push product                           │
│    - JUMP: Set PC                                       │
│    - CALL: Push PC, jump to function                    │
│                                                         │
│  Constraint: Each instruction transition is valid       │
└─────────────────────────────────────────────────────────┘
            ↓
  Proof: "Program executed correctly, output = X"
```

### Implementation Plan

#### Phase 1: TinyVM (Minimal Implementation)

**Goal:** Build smallest possible VM to understand concepts

**Instruction Set (10 instructions):**
```rust
enum Instruction {
    PUSH(u64),      // Push constant to stack
    POP,            // Remove top of stack
    ADD,            // Pop 2, push sum
    MUL,            // Pop 2, push product
    DUP,            // Duplicate top of stack
    SWAP,           // Swap top 2 stack elements
    JUMP(u64),      // Set PC to address
    JUMP_IF_ZERO(u64), // Jump if top is zero
    CALL(u64),      // Call function at address
    RETURN,         // Return from function
}
```

**VM State:**
```rust
struct VMState {
    pc: u64,                    // Program counter
    stack: Vec<Fr>,             // Stack (max depth: 64)
    memory: Vec<Fr>,            // Memory (optional, start simple)
}
```

**Circuit Design:**
```rust
// For each instruction, create constraints:
// - PC increments correctly
// - Stack operations are valid
// - Instruction semantics are enforced

template VMStep() {
    signal input current_pc;
    signal input current_stack[64];
    signal input instruction;
    signal input instruction_arg;
    
    signal output next_pc;
    signal output next_stack[64];
    
    // Instruction decoding
    component decoder = InstructionDecoder();
    decoder.instruction <== instruction;
    
    // Execute instruction
    component executor = InstructionExecutor();
    executor.pc <== current_pc;
    executor.stack <== current_stack;
    executor.opcode <== decoder.opcode;
    executor.arg <== instruction_arg;
    
    next_pc <== executor.next_pc;
    next_stack <== executor.next_stack;
}
```

**Example Program:**
```rust
// Compute: x² + 2x + 1
let program = vec![
    Instruction::PUSH(1),   // Load x (assume in memory[0])
    Instruction::DUP,         // x, x
    Instruction::MUL,        // x²
    Instruction::PUSH(2),    // x², 2
    Instruction::PUSH(1),    // x², 2, x (from memory)
    Instruction::MUL,        // x², 2x
    Instruction::ADD,        // x² + 2x
    Instruction::PUSH(1),    // x² + 2x, 1
    Instruction::ADD,        // x² + 2x + 1
    Instruction::RETURN,
];
```

**Expected Constraints:**
- Per instruction: ~100-500 constraints
- 10-instruction program: ~1K-5K constraints
- Simple but demonstrates concept

#### Phase 2: Enhanced VM

**Add:**
- Memory access (load/store)
- More instructions (SUB, DIV, MOD, etc.)
- Function calls with stack frames
- Loops (convert to unrolled)

**Complexity:** ~10K-50K constraints per program

#### Phase 3: RISC-V Subset (Advanced)

**Goal:** Implement subset of RISC-V ISA

**Instructions:**
- Basic arithmetic (ADD, SUB, MUL)
- Load/store (LW, SW)
- Branches (BEQ, BNE)
- Jumps (JAL, JALR)

**Complexity:** ~100K-1M constraints per instruction

### Recommended Structure

```
04-zkvm/
├── 04-01-tiny-vm/
│   ├── src/
│   │   ├── vm.rs                # VM state machine
│   │   ├── instructions.rs     # Instruction set
│   │   ├── compiler.rs          # Program → bytecode
│   │   └── circuit.rs           # VM → circuit compiler
│   ├── examples/
│   │   ├── fibonacci.rs         # Fibonacci in VM
│   │   ├── factorial.rs         # Factorial in VM
│   │   └── simple_math.rs      # Basic arithmetic
│   ├── tests/
│   │   └── vm_test.rs
│   └── README.md
│
├── 04-02-enhanced-vm/
│   ├── src/
│   │   ├── memory.rs            # Memory management
│   │   ├── functions.rs         # Function calls
│   │   └── loops.rs             # Loop unrolling
│   └── examples/
│       └── recursive.rs         # Recursive functions
│
├── 04-03-risc-v-subset/         # Advanced (optional)
│   └── README.md                # RISC-V implementation guide
│
└── README.md                    # zkVM overview
```

### Learning Resources

**Papers:**
- "zkVM: Zero-Knowledge Virtual Machine" - Various implementations
- RISC-V Specification (subset)
- "TinyRAM" - Early zkVM design

**Implementations:**
- [RISC Zero](https://www.risczero.com/) - RISC-V zkVM
- [zkWASM](https://github.com/DelphinusLab/zkWASM) - WebAssembly zkVM
- [Miden VM](https://github.com/0xPolygonMiden/miden-vm) - Stack-based zkVM

**Tutorials:**
- RISC Zero docs - Good introduction
- 0xPARC zkVM course (if available)

---

## Implementation Priority

### For Architect Roles

**Must Have:**
1. ✅ **Simple Recursion** (2-proof aggregator)
   - Shows understanding of verifier circuits
   - Demonstrates proof compression
   - Time: 1-2 weeks

2. ✅ **TinyVM** (10-instruction VM)
   - Shows VM → circuit compilation
   - Demonstrates state machine design
   - Time: 2-3 weeks

**Nice to Have:**
3. Batch aggregator (N proofs)
4. Enhanced VM (memory, functions)
5. RISC-V subset (advanced)

### Time Investment

**Minimum Viable:**
- Recursion: 1-2 weeks (simple aggregator)
- zkVM: 2-3 weeks (TinyVM)
- **Total: 3-5 weeks**

**Comprehensive:**
- Recursion: 3-4 weeks (batch + optimizations)
- zkVM: 4-6 weeks (enhanced + examples)
- **Total: 7-10 weeks**

---

## Success Criteria

### Recursion

✅ **Complete when:**
- Can verify 2 Groth16 proofs in PLONK circuit
- Generate recursive proof successfully
- Demonstrate gas savings (2 proofs → 1 proof)
- Document verifier circuit design

### zkVM

✅ **Complete when:**
- Can compile simple program to bytecode
- Execute bytecode in VM circuit
- Generate proof of correct execution
- Support at least 5 different programs
- Document VM architecture

---

## Next Actions

1. **Choose Framework:**
   - Recursion: Halo2 (cycle-friendly) or Plonky2
   - zkVM: Halo2 (flexibility) or Circom (simpler)

2. **Start Small:**
   - Recursion: 2-proof aggregator first
   - zkVM: 5-instruction VM first

3. **Iterate:**
   - Add features incrementally
   - Document learnings
   - Benchmark performance

4. **Showcase:**
   - Update README with new sections
   - Add to portfolio
   - Write blog post about implementation
