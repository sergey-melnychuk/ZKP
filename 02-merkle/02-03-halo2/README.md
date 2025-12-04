# Halo2 Merkle Proof - Complete Implementation Guide

**WARNING:** This is a complex, low-level PLONK implementation. If you want something easier, use Circom (Groth16) or Noir (high-level PLONK).

**What this is:** A production-grade Merkle tree membership proof in Halo2 with full conditional path selection.

**Why it's complicated:** Halo2 is the lowest-level ZK framework - you implement every constraint manually.

---

## 🎯 What This Code Does

**Goal:** Prove you know a secret that's in a Merkle tree, without revealing the secret.

**Inputs:**
- **Private:** `secret`, `siblings[3]`, `path_indices[3]`
- **Public:** `root`, `nullifier`

**Verification:**
1. Compute `leaf = Poseidon(secret)`
2. Compute `nullifier = Poseidon(secret)` and check it matches public input
3. Climb tree using siblings and path_indices
4. Verify computed root matches public root

---

## 📁 File Structure

```
src/lib.rs
├── MerkleConfig        - Circuit configuration (columns, gates)
├── MerkleCircuit       - Circuit implementation
├── Circuit trait impl  - configure() + synthesize()
└── Tests              - 3 comprehensive tests
```

---

## 🧩 Code Breakdown - Every Single Piece Explained

### Part 1: Dependencies

```rust
use ff::Field;
```
**What:** Field arithmetic trait (addition, multiplication in finite fields)  
**Why:** All ZK operations happen in finite fields (modular arithmetic)

```rust
use halo2_gadgets::poseidon::{
    primitives::{ConstantLength, P128Pow5T3 as OrchardNullifier},
    Hash, Pow5Chip, Pow5Config,
};
```
**What:** Poseidon hash implementation for Halo2  
**Why:** We need cryptographic hashing inside the circuit  
**P128Pow5T3:** Specific Poseidon parameters (128-bit security, power-of-5 S-box, width-3)  
**OrchardNullifier:** Alias used by Zcash Orchard protocol (battle-tested parameters)

```rust
use halo2_proofs::{
    circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Expression, Fixed, Instance, Selector},
    poly::Rotation,
};
```
**What:** Core Halo2 types  
**Why:**
- `Advice` - private data columns
- `Instance` - public input columns  
- `Selector` - enable/disable gates
- `Layouter` - assigns values to circuit
- `Expression` - polynomial expressions for constraints

```rust
use pasta_curves::pallas;
```
**What:** Pallas elliptic curve (part of Pasta curves)  
**Why:** Halo2 uses Pallas for the proof system  
**Note:** `pallas::Base` = the field we work in

---

### Part 2: Circuit Configuration

```rust
#[derive(Clone, Debug)]
struct MerkleConfig {
    advices: [Column<Advice>; 10],
    poseidon_config: Pow5Config<pallas::Base, 3, 2>,
    instance: Column<Instance>,
    selector: Selector,
}
```

**What is this?** The "blueprint" for our circuit - defines what columns and gates we have.

**advices: [Column<Advice>; 10]**
- 10 columns for holding private data
- Think of columns as vertical slots in a table
- Why 10? Poseidon needs 3 columns + we need extras for our logic

**poseidon_config: Pow5Config<pallas::Base, 3, 2>**
- Configuration for Poseidon hash chip
- `3` = width (internal state size)
- `2` = rate (how many inputs per hash)

**instance: Column<Instance>**
- Column for public inputs (root, nullifier)
- Verifier can see these values

**selector: Selector**
- On/off switch for our custom swap gate
- Enables gate only in rows where we need it

---

### Part 3: Custom Swap Gate

```rust
impl MerkleConfig {
    fn configure_swap_gate(&self, meta: &mut ConstraintSystem<pallas::Base>) {
```

**What is this?** Custom constraint that implements conditional selection.

**Problem:** In Merkle trees, we need to swap current/sibling based on path_index:
- If `path_index = 0`: we're on the left, so `(left=current, right=sibling)`
- If `path_index = 1`: we're on the right, so `(left=sibling, right=current)`

**Why can't we use if/else?** ZK circuits don't have branching! Everything must be polynomial equations.

```rust
meta.create_gate("swap", |meta| {
    let s = meta.query_selector(self.selector);
```
**What:** Create a gate named "swap", query if selector is enabled  
**Why:** Gate only applies when selector = 1

```rust
    let current = meta.query_advice(self.advices[5], Rotation::cur());
    let sibling = meta.query_advice(self.advices[6], Rotation::cur());
    let path_index = meta.query_advice(self.advices[7], Rotation::cur());
    let left = meta.query_advice(self.advices[8], Rotation::cur());
    let right = meta.query_advice(self.advices[9], Rotation::cur());
```
**What:** Read values from columns 5-9 in the current row  
**Why:** These columns hold: current hash, sibling, path choice, and outputs (left, right)

```rust
    // Constrain path_index is binary: path_index * (1 - path_index) = 0
    let bool_check = path_index.clone() * (Expression::Constant(pallas::Base::ONE) - path_index.clone());
```
**What:** Polynomial that's 0 only if path_index ∈ {0, 1}  
**Why:** Prevents cheating with path_index = 0.5 or other invalid values  
**Math:** If x=0: 0*(1-0)=0 ✓, if x=1: 1*(1-1)=0 ✓, if x=0.5: 0.5*0.5=0.25 ✗

```rust
    // left = current * (1 - path_index) + sibling * path_index
    // right = sibling * (1 - path_index) + current * path_index
    
    let expected_left = current.clone() * (Expression::Constant(pallas::Base::ONE) - path_index.clone())
        + sibling.clone() * path_index.clone();
    let expected_right = sibling.clone() * (Expression::Constant(pallas::Base::ONE) - path_index.clone())
        + current.clone() * path_index;
```
**What:** Polynomial expressions for conditional selection  
**Why:** This implements if/else using arithmetic:
- When path_index=0: `left = current*1 + sibling*0 = current`, `right = sibling*1 + current*0 = sibling`
- When path_index=1: `left = current*0 + sibling*1 = sibling`, `right = sibling*0 + current*1 = current`

**This is the core trick of ZK circuits:** Implement branching with arithmetic!

```rust
    vec![
        s.clone() * bool_check,
        s.clone() * (left - expected_left),
        s * (right - expected_right),
    ]
});
```
**What:** Return 3 constraints (polynomials that must equal 0)  
**Why:** 
1. `path_index` must be binary (0 or 1)
2. `left` must equal the computed expected_left
3. `right` must equal the computed expected_right

**All multiplied by selector `s`:** Constraints only enforced when selector is on.

---

### Part 4: Circuit Structure

```rust
#[derive(Clone, Debug, Default)]
struct MerkleCircuit {
    secret: Value<pallas::Base>,
    siblings: Value<[pallas::Base; 3]>,
    path_indices: Value<[bool; 3]>,
}
```

**What:** The circuit inputs  
**Why `Value<T>`?** Halo2's type that can be:
- `Value::known(x)` - when we have a witness (prover)
- `Value::unknown()` - when we don't have a witness (setup)

**No root or nullifier here?** Those are public inputs, passed separately in tests.

---

### Part 5: Circuit Trait Implementation

```rust
impl Circuit<pallas::Base> for MerkleCircuit {
    type Config = MerkleConfig;
    type FloorPlanner = SimpleFloorPlanner;
```

**What:** Implement the Circuit trait for our struct  
**Config:** What configuration type we use  
**FloorPlanner:** How to lay out the circuit (SimpleFloorPlanner = basic strategy)

```rust
    fn without_witnesses(&self) -> Self {
        Self::default()
    }
```
**What:** Return circuit with no witnesses (all unknown)  
**Why:** Used during setup phase when we don't have actual values yet

```rust
    fn configure(meta: &mut ConstraintSystem<pallas::Base>) -> Self::Config {
```
**What:** Define the circuit structure (columns, gates, constraints)  
**When called:** Once, during circuit compilation  
**Note:** This is static - doesn't use `self`, defines structure not values

```rust
        let advices = [
            meta.advice_column(),
            meta.advice_column(),
            // ... 10 total
        ];
```
**What:** Create 10 advice columns  
**Why 10?**
- Columns 0-3: Poseidon internal state + result
- Column 4: Spare for assigning siblings
- Columns 5-9: Used by swap gate (current, sibling, path_index, left, right)

```rust
        for advice in advices.iter() {
            meta.enable_equality(*advice);
        }
```
**What:** Allow copy constraints between cells in these columns  
**Why:** We need to "copy" values between regions (e.g., result of one hash becomes input to next)

```rust
        let instance = meta.instance_column();
        meta.enable_equality(instance);
```
**What:** Create public input column and enable equality  
**Why:** We'll constrain root and nullifier to match public inputs

```rust
        let lagrange_coeffs = [
            meta.fixed_column(),
            // ... 8 total
        ];
        meta.enable_constant(lagrange_coeffs[0]);
```
**What:** Fixed columns for Poseidon round constants  
**Why:** Poseidon hash needs hardcoded constants, these go in Fixed columns  
**enable_constant:** Allows assigning constant values to these columns

```rust
        let poseidon_config = Pow5Chip::configure::<OrchardNullifier>(
            meta,
            advices[0..3].try_into().unwrap(),      // state columns
            advices[3],                              // result column
            lagrange_coeffs[0..3].try_into().unwrap(), // full round constants
            lagrange_coeffs[3..6].try_into().unwrap(), // partial round constants
        );
```
**What:** Configure the Poseidon chip with our columns  
**Why:** Poseidon needs to know which columns to use for its internal state and constants

```rust
        let selector = meta.selector();
```
**What:** Create a selector for our swap gate  
**Why:** We'll turn this on only in rows where we do path swapping

```rust
        let config = MerkleConfig { /* ... */ };
        config.configure_swap_gate(meta);
        config
```
**What:** Create our config, configure the swap gate, return config  
**Why:** This tells Halo2 what our circuit looks like

---

### Part 6: Synthesis (The Actual Proof Logic)

```rust
    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<pallas::Base>,
    ) -> Result<(), Error> {
```

**What:** Fill the circuit with actual values (witness generation)  
**When called:** When creating a proof (prover has the witness)  
**layouter:** Tool for assigning values to cells

#### Step 1: Assign Secret

```rust
        let secret = layouter.assign_region(
            || "load secret",
            |mut region| {
                region.assign_advice(|| "secret", config.advices[0], 0, || self.secret)
            },
        )?;
```

**What:** Put the secret value into column 0, row 0  
**Why:** We need the secret in a cell to use it  
**Returns:** `AssignedCell<pallas::Base, pallas::Base>` - a reference to that cell

**assign_region pattern:**
```
layouter.assign_region(|| "label", |mut region| {
    // do assignments
    region.assign_advice(|| "label", column, row, || value)
})
```
This is Halo2's way of organizing the circuit into logical regions.

#### Step 2: Compute Leaf

```rust
        let leaf = {
            let poseidon_chip = Pow5Chip::construct(config.poseidon_config.clone());
            let hasher = Hash::<_, _, OrchardNullifier, ConstantLength<1>, 3, 2>::init(
                poseidon_chip,
                layouter.namespace(|| "leaf hasher"),
            )?;
            hasher.hash(layouter.namespace(|| "hash leaf"), [secret.clone()])?
        };
```

**What:** Compute `leaf = Poseidon(secret)`

**Breaking it down:**

```rust
let poseidon_chip = Pow5Chip::construct(config.poseidon_config.clone());
```
**What:** Create an instance of the Poseidon chip  
**Why:** We need this to perform Poseidon hashing

```rust
let hasher = Hash::<_, _, OrchardNullifier, ConstantLength<1>, 3, 2>::init(
    poseidon_chip,
    layouter.namespace(|| "leaf hasher"),
)?;
```
**What:** Initialize a Poseidon hasher for 1 input  
**Type parameters:**
- `OrchardNullifier` - Poseidon spec (which parameters to use)
- `ConstantLength<1>` - hashing exactly 1 input
- `3, 2` - width=3, rate=2 (Poseidon params)

**namespace:** Creates a sub-namespace for this hasher's cells

```rust
hasher.hash(layouter.namespace(|| "hash leaf"), [secret.clone()])?
```
**What:** Actually perform the hash, get result as AssignedCell  
**Why clone?** We'll use secret again for nullifier

#### Step 3: Compute Nullifier

```rust
        let nullifier = {
            let poseidon_chip = Pow5Chip::construct(config.poseidon_config.clone());
            let hasher = Hash::<_, _, OrchardNullifier, ConstantLength<1>, 3, 2>::init(
                poseidon_chip,
                layouter.namespace(|| "nullifier hasher"),
            )?;
            hasher.hash(layouter.namespace(|| "hash nullifier"), [secret])?
        };
```

**What:** Compute `nullifier = Poseidon(secret)` (same as leaf)  
**Why separate?** For clarity and to expose as separate public input

#### Step 4: Climb Merkle Tree (The Complex Part)

```rust
        let mut current = leaf;
        for i in 0..3 {
```
**What:** Start with leaf, iterate 3 times (tree depth = 3)

```rust
            let (left, right) = layouter.assign_region(
                || format!("swap level {}", i),
                |mut region| {
                    config.selector.enable(&mut region, 0)?;
```
**What:** Create a region for this level's swap operation  
**enable selector:** Turn on the swap gate in row 0 of this region

```rust
                    let current_copy = current.copy_advice(
                        || "current",
                        &mut region,
                        config.advices[5],
                        0,
                    )?;
```
**What:** Copy the current hash into column 5, row 0 of this region  
**Why copy?** We need it in this region to use with the swap gate  
**copy_advice:** Creates equality constraint (original cell = new cell)

```rust
                    let sibling = self.siblings.map(|siblings| siblings[i]);
                    let sibling_cell = region.assign_advice(
                        || "sibling",
                        config.advices[6],
                        0,
                        || sibling,
                    )?;
```
**What:** Assign sibling[i] to column 6, row 0  
**Why:** Swap gate needs sibling in column 6

```rust
                    let path_index = self.path_indices.map(|indices| {
                        if indices[i] {
                            pallas::Base::ONE
                        } else {
                            pallas::Base::ZERO
                        }
                    });
                    region.assign_advice(
                        || "path_index",
                        config.advices[7],
                        0,
                        || path_index,
                    )?;
```
**What:** Convert path_indices[i] (bool) to Field element (0 or 1), assign to column 7  
**Why:** Our swap gate works with field elements, not booleans

```rust
                    let left_value = self.path_indices.zip(sibling).zip(current_copy.value().copied())
                        .map(|((indices, sib), cur)| {
                            if indices[i] {
                                sib  // If right path, sibling goes left
                            } else {
                                cur  // If left path, current goes left
                            }
                        });
```
**What:** Compute what left should be based on path_index  
**Why:** We need to assign the correct values to left and right cells

**Breaking down the chain:**
- `self.path_indices` - Value<[bool; 3]>
- `.zip(sibling)` - Value<([bool; 3], pallas::Base)>
- `.zip(current_copy.value().copied())` - Value<(([bool; 3], pallas::Base), pallas::Base)>
- `.map(...)` - Value<pallas::Base> (the computed left value)

```rust
                    let right_value = self.path_indices.zip(sibling).zip(current_copy.value().copied())
                        .map(|((indices, sib), cur)| {
                            if indices[i] {
                                cur  // If right path, current goes right
                            } else {
                                sib  // If left path, sibling goes right
                            }
                        });
```
**What:** Similarly compute right value

```rust
                    let left_cell = region.assign_advice(
                        || "left",
                        config.advices[8],
                        0,
                        || left_value,
                    )?;

                    let right_cell = region.assign_advice(
                        || "right",
                        config.advices[9],
                        0,
                        || right_value,
                    )?;

                    Ok((left_cell, right_cell))
                },
            )?;
```
**What:** Assign computed left and right to columns 8 and 9  
**Why:** Swap gate will check these match the constraint equations  
**Return:** The left and right cells to use for hashing

**At this point in the region (row 0):**
- Column 5: current hash
- Column 6: sibling
- Column 7: path_index (0 or 1)
- Column 8: left (computed)
- Column 9: right (computed)

**The swap gate checks:**
- path_index is binary
- left = current*(1-path_index) + sibling*path_index
- right = sibling*(1-path_index) + current*path_index

#### Step 5: Hash to Get Parent

```rust
            let poseidon_chip = Pow5Chip::construct(config.poseidon_config.clone());
            let hasher = Hash::<_, _, OrchardNullifier, ConstantLength<2>, 3, 2>::init(
                poseidon_chip,
                layouter.namespace(|| format!("tree hasher {}", i)),
            )?;
            current = hasher.hash(
                layouter.namespace(|| format!("hash level {}", i)),
                [left, right],
            )?;
        }
```
**What:** Hash left and right to get parent, update current  
**Note:** `ConstantLength<2>` because hashing 2 inputs now

**After 3 iterations:** current = root of the tree

#### Step 6: Constrain Public Inputs

```rust
        layouter.constrain_instance(current.cell(), config.instance, 0)?;
```
**What:** Force current (computed root) to equal public input at instance[0]  
**Why:** This proves we computed the correct root

```rust
        layouter.constrain_instance(nullifier.cell(), config.instance, 1)?;
```
**What:** Force nullifier to equal public input at instance[1]  
**Why:** This proves we computed the correct nullifier from the secret

**If either constraint fails:** Proof won't verify!

---

## 🧪 Tests Explained

### Test 1: Left Path

```rust
let path_indices = [false; 3]; // All left
```
**What:** Secret is at index 0 (leftmost leaf)  
**Siblings:** All zeros (empty tree except our leaf)  
**Path:** Left, left, left up the tree

### Test 2: Right Path

```rust
let path_indices = [true; 3]; // All right
```
**What:** Secret is at index 7 (rightmost leaf)  
**Siblings:** Non-zero (there are other leaves)  
**Path:** Right, right, right up the tree

### Test 3: Mixed Path

```rust
let path_indices = [false, true, false]; // left, right, left
```
**What:** Secret is at some middle position  
**Path:** Zigzag through the tree

**All tests compute expected root externally and verify circuit computes same root.**

---

## 🎓 Key Concepts Explained

### 1. Why No If/Else in ZK Circuits?

**Problem:** Circuits are polynomial equations. Branching breaks this.

**Solution:** Use arithmetic to simulate branching:
```
result = condition * value_if_true + (1 - condition) * value_if_false
```

If condition=1: result = 1*value_if_true + 0*value_if_false = value_if_true  
If condition=0: result = 0*value_if_true + 1*value_if_false = value_if_false

### 2. Why So Many Columns?

**Each operation needs columns:**
- Poseidon: 4 columns (3 state + 1 result)
- Swap gate: 5 columns (current, sibling, path_index, left, right)
- Extra: For intermediate values

**Total:** 10 columns to have enough space

### 3. What Are Regions?

**Regions:** Logical groupings of rows where related operations happen

**Why useful:**
- Organization (easier to debug)
- Namespace management
- Can reuse same columns in different regions

### 4. What is Copy Advice?

**Problem:** Value computed in one region, needed in another

**Solution:** `copy_advice` creates equality constraint:
```
cell_A = cell_B
```

**Implementation:** Uses permutation argument (core PLONK feature)

### 5. Why Poseidon Over SHA256?

**In ZK circuits:**
- SHA256: ~25,000 constraints
- Poseidon: ~150-200 constraints (R1CS) or ~2 opcodes (PLONK custom gates)

**Reason:** Poseidon designed for ZK (arithmetic-friendly operations)

---

## 🔍 Common Gotchas

### 1. clone() on Poseidon Config

```rust
let poseidon_chip = Pow5Chip::construct(config.poseidon_config.clone());
```

**Why needed?** We create multiple Poseidon chips (leaf, nullifier, each level)  
**Alternative:** Could use single chip, more complex code

### 2. Value::map() Chains

```rust
self.path_indices.zip(sibling).zip(current).map(...)
```

**What's happening?** Working inside Value monad  
**Why?** Values might be unknown (during setup), so we need monadic operations

### 3. Warnings About Dead Code

**Why?** Compiler doesn't see Halo2 framework calling our methods  
**Solution:** Add `#[allow(dead_code)]` or ignore - code is fine

---

## 🚀 How to Use This Code

### 1. Add to Your Project

```toml
[dependencies]
halo2_proofs = "0.3"
halo2_gadgets = "0.3"
ff = "0.13"
group = "0.13"
pasta_curves = "0.5"
```

### 2. Run Tests

```bash
cargo test -- --nocapture
```

### 3. Generate Real Proof (Beyond This Code)

```rust
use halo2_proofs::plonk::{create_proof, keygen_pk, keygen_vk};
use halo2_proofs::poly::commitment::Params;
use rand::rngs::OsRng;

// Setup
let params = Params::new(11); // k=11
let empty_circuit = MerkleCircuit::default();
let vk = keygen_vk(&params, &empty_circuit)?;
let pk = keygen_pk(&params, vk, &empty_circuit)?;

// Prove
let circuit = MerkleCircuit { /* real values */ };
let mut transcript = Blake2bWrite::init(vec![]);
create_proof(&params, &pk, &[circuit], &[&[&[root, nullifier]]], OsRng, &mut transcript)?;
let proof = transcript.finalize();

// Verify
let strategy = SingleVerifier::new(&params);
let mut transcript = Blake2bRead::init(&proof[..]);
verify_proof(&params, pk.get_vk(), strategy, &[&[&[root, nullifier]]], &mut transcript)?;
```

---

## 📚 Further Reading

**If you want to understand Halo2 deeply:**

1. **Halo2 Book:** https://zcash.github.io/halo2/
2. **PLONK Paper:** "PLONK: Permutations over Lagrange-bases for Oecumenical Noninteractive arguments of Knowledge"
3. **Poseidon Paper:** "Poseidon: A New Hash Function for Zero-Knowledge Proof Systems"
4. **Zcash Orchard:** https://github.com/zcash/orchard (production Halo2 code)

**If you want easier alternatives:**

1. **Circom (Groth16):** High-level, mature, but circuit-specific setup
2. **Noir (PLONK):** Rust-like syntax, much easier than Halo2
3. **ZoKrates:** Python-like, good for learning

---

## ❓ FAQ

### Q: Why is this 250 lines when Noir is 30 lines?

**A:** Halo2 is low-level. You implement every constraint manually. It's like assembly vs high-level language.

### Q: When should I use Halo2?

**A:** When you need:
- Maximum performance
- Custom gates for your specific use case
- Production-grade code (Zcash uses it)
- Deep control over circuit layout

### Q: Can I just use the gadgets library?

**A:** Yes! halo2_gadgets provides Poseidon, ECDSA, etc. But you still need to understand circuits.

### Q: Why pasta_curves and not bn254?

**A:** Halo2 uses Pasta curves (Pallas/Vesta) for proof recursion. You can use bn254 but need different dependencies.

### Q: How do I debug "constraint not satisfied"?

**A:** Use MockProver with `.verify()`. It shows which constraints fail and in which rows.

```rust
match prover.verify() {
    Ok(()) => println!("Success!"),
    Err(e) => println!("Failed: {:?}", e),
}
```

---

## 🎯 Summary

**What you built:** Production-grade Merkle proof in Halo2

**Key achievements:**
- ✅ Custom swap gate with binary constraints
- ✅ Proper path selection (left/right conditional)
- ✅ Poseidon hashing with gadgets
- ✅ Public input constraints
- ✅ Comprehensive tests

**Complexity level:** Advanced (this is hard stuff!)

**Line count:** ~250 lines (vs ~30 in Noir, ~45 in Circom)

**Why it's worth it:** You now understand PLONK at the lowest level. This is production-grade knowledge.

---

**Final note:** If you ever think "this is too complicated", you're right. That's why higher-level frameworks exist. But knowing how to work at this level makes you a ZK expert.
