use ff::Field;
use halo2_gadgets::poseidon::{
    primitives::{ConstantLength, P128Pow5T3 as OrchardNullifier},
    Hash, Pow5Chip, Pow5Config,
};
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{
        Advice, Circuit, Column, ConstraintSystem, Error, Expression, Instance, Selector,
    },
    poly::Rotation,
};
use pasta_curves::pallas;

#[allow(dead_code)]
#[derive(Clone, Debug)]
struct MerkleConfig {
    advices: [Column<Advice>; 10],
    poseidon_config: Pow5Config<pallas::Base, 3, 2>,
    instance: Column<Instance>,
    selector: Selector,
}

impl MerkleConfig {
    fn configure_swap_gate(&self, meta: &mut ConstraintSystem<pallas::Base>) {
        meta.create_gate("swap", |meta| {
            let s = meta.query_selector(self.selector);

            // current, sibling, path_index, left, right
            let current = meta.query_advice(self.advices[5], Rotation::cur());
            let sibling = meta.query_advice(self.advices[6], Rotation::cur());
            let path_index = meta.query_advice(self.advices[7], Rotation::cur());
            let left = meta.query_advice(self.advices[8], Rotation::cur());
            let right = meta.query_advice(self.advices[9], Rotation::cur());

            // Constrain path_index is binary: path_index * (1 - path_index) = 0
            let bool_check =
                path_index.clone() * (Expression::Constant(pallas::Base::ONE) - path_index.clone());

            // If path_index = 0: left = current, right = sibling
            // If path_index = 1: left = sibling, right = current
            // left = current * (1 - path_index) + sibling * path_index
            // right = sibling * (1 - path_index) + current * path_index

            let expected_left = current.clone()
                * (Expression::Constant(pallas::Base::ONE) - path_index.clone())
                + sibling.clone() * path_index.clone();
            let expected_right = sibling.clone()
                * (Expression::Constant(pallas::Base::ONE) - path_index.clone())
                + current.clone() * path_index;

            vec![
                s.clone() * bool_check,
                s.clone() * (left - expected_left),
                s * (right - expected_right),
            ]
        });
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
struct MerkleCircuit {
    secret: Value<pallas::Base>,
    siblings: Value<[pallas::Base; 3]>,
    path_indices: Value<[bool; 3]>,
}

impl Circuit<pallas::Base> for MerkleCircuit {
    type Config = MerkleConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<pallas::Base>) -> Self::Config {
        let advices = [
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
        ];

        for advice in advices.iter() {
            meta.enable_equality(*advice);
        }

        let instance = meta.instance_column();
        meta.enable_equality(instance);

        let lagrange_coeffs = [
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
        ];
        meta.enable_constant(lagrange_coeffs[0]);

        let poseidon_config = Pow5Chip::configure::<OrchardNullifier>(
            meta,
            advices[0..3].try_into().unwrap(),
            advices[3],
            lagrange_coeffs[0..3].try_into().unwrap(),
            lagrange_coeffs[3..6].try_into().unwrap(),
        );

        let selector = meta.selector();

        let config = MerkleConfig {
            advices,
            poseidon_config,
            instance,
            selector,
        };

        config.configure_swap_gate(meta);

        config
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<pallas::Base>,
    ) -> Result<(), Error> {
        // Load secret
        let secret = layouter.assign_region(
            || "load secret",
            |mut region| region.assign_advice(|| "secret", config.advices[0], 0, || self.secret),
        )?;

        // Compute leaf = hash(secret)
        let leaf = {
            let poseidon_chip = Pow5Chip::construct(config.poseidon_config.clone());
            let hasher = Hash::<_, _, OrchardNullifier, ConstantLength<1>, 3, 2>::init(
                poseidon_chip,
                layouter.namespace(|| "leaf hasher"),
            )?;
            hasher.hash(layouter.namespace(|| "hash leaf"), [secret.clone()])?
        };

        // Compute nullifier = hash(secret)
        let nullifier = {
            let poseidon_chip = Pow5Chip::construct(config.poseidon_config.clone());
            let hasher = Hash::<_, _, OrchardNullifier, ConstantLength<1>, 3, 2>::init(
                poseidon_chip,
                layouter.namespace(|| "nullifier hasher"),
            )?;
            hasher.hash(layouter.namespace(|| "hash nullifier"), [secret])?
        };

        // Climb tree with proper path selection
        let mut current = leaf;

        for i in 0..3 {
            let (left, right) = layouter.assign_region(
                || format!("swap level {}", i),
                |mut region| {
                    config.selector.enable(&mut region, 0)?;

                    // Copy current into region
                    let current_copy =
                        current.copy_advice(|| "current", &mut region, config.advices[5], 0)?;

                    // Assign sibling
                    let sibling = self.siblings.map(|siblings| siblings[i]);
                    let _sibling_cell =
                        region.assign_advice(|| "sibling", config.advices[6], 0, || sibling)?;

                    // Assign path_index (as Field element: 0 or 1)
                    let path_index = self.path_indices.map(|indices| {
                        if indices[i] {
                            pallas::Base::ONE
                        } else {
                            pallas::Base::ZERO
                        }
                    });
                    region.assign_advice(|| "path_index", config.advices[7], 0, || path_index)?;

                    // Compute left and right
                    let left_value = self
                        .path_indices
                        .zip(sibling)
                        .zip(current_copy.value().copied())
                        .map(|((indices, sib), cur)| if indices[i] { sib } else { cur });

                    let right_value = self
                        .path_indices
                        .zip(sibling)
                        .zip(current_copy.value().copied())
                        .map(|((indices, sib), cur)| if indices[i] { cur } else { sib });

                    let left_cell =
                        region.assign_advice(|| "left", config.advices[8], 0, || left_value)?;

                    let right_cell =
                        region.assign_advice(|| "right", config.advices[9], 0, || right_value)?;

                    Ok((left_cell, right_cell))
                },
            )?;

            // Hash left and right to get parent
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

        // Expose root
        layouter.constrain_instance(current.cell(), config.instance, 0)?;

        // Expose nullifier
        layouter.constrain_instance(nullifier.cell(), config.instance, 1)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use halo2_gadgets::poseidon::primitives::{self as poseidon, P128Pow5T3};
    use halo2_proofs::{dev::MockProver, pasta::Fp};

    #[test]
    fn test_merkle_left_path() {
        let k = 11;
        let secret = Fp::from(12345);

        // Compute expected values
        let leaf = poseidon::Hash::<_, P128Pow5T3, ConstantLength<1>, 3, 2>::init().hash([secret]);
        let siblings = [Fp::zero(); 3];
        let path_indices = [false; 3]; // All left

        // Compute root
        let mut current = leaf;
        for i in 0..3 {
            let (left, right) = if path_indices[i] {
                (siblings[i], current)
            } else {
                (current, siblings[i])
            };
            current = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
                .hash([left, right]);
        }
        let root = current;

        let nullifier =
            poseidon::Hash::<_, P128Pow5T3, ConstantLength<1>, 3, 2>::init().hash([secret]);

        // Create circuit
        let circuit = MerkleCircuit {
            secret: Value::known(secret),
            siblings: Value::known(siblings),
            path_indices: Value::known(path_indices),
        };

        let public_inputs = vec![root, nullifier];
        let prover = MockProver::run(k, &circuit, vec![public_inputs]).unwrap();
        assert_eq!(prover.verify(), Ok(()));

        println!("✅ Left path works!");
    }

    #[test]
    fn test_merkle_right_path() {
        let k = 11;
        let secret = Fp::from(67890);

        // Compute expected values
        let leaf = poseidon::Hash::<_, P128Pow5T3, ConstantLength<1>, 3, 2>::init().hash([secret]);
        let siblings = [Fp::from(111), Fp::from(222), Fp::from(333)];
        let path_indices = [true; 3]; // All right

        // Compute root
        let mut current = leaf;
        for i in 0..3 {
            let (left, right) = if path_indices[i] {
                (siblings[i], current)
            } else {
                (current, siblings[i])
            };
            current = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
                .hash([left, right]);
        }
        let root = current;

        let nullifier =
            poseidon::Hash::<_, P128Pow5T3, ConstantLength<1>, 3, 2>::init().hash([secret]);

        // Create circuit
        let circuit = MerkleCircuit {
            secret: Value::known(secret),
            siblings: Value::known(siblings),
            path_indices: Value::known(path_indices),
        };

        let public_inputs = vec![root, nullifier];
        let prover = MockProver::run(k, &circuit, vec![public_inputs]).unwrap();
        assert_eq!(prover.verify(), Ok(()));

        println!("✅ Right path works!");
    }

    #[test]
    fn test_merkle_mixed_path() {
        let k = 11;
        let secret = Fp::from(99999);

        let leaf = poseidon::Hash::<_, P128Pow5T3, ConstantLength<1>, 3, 2>::init().hash([secret]);
        let siblings = [Fp::from(10), Fp::from(20), Fp::from(30)];
        let path_indices = [false, true, false]; // left, right, left

        // Compute root
        let mut current = leaf;
        for i in 0..3 {
            let (left, right) = if path_indices[i] {
                (siblings[i], current)
            } else {
                (current, siblings[i])
            };
            current = poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
                .hash([left, right]);
        }
        let root = current;

        let nullifier =
            poseidon::Hash::<_, P128Pow5T3, ConstantLength<1>, 3, 2>::init().hash([secret]);

        let circuit = MerkleCircuit {
            secret: Value::known(secret),
            siblings: Value::known(siblings),
            path_indices: Value::known(path_indices),
        };

        let public_inputs = vec![root, nullifier];
        let prover = MockProver::run(k, &circuit, vec![public_inputs]).unwrap();
        assert_eq!(prover.verify(), Ok(()));

        println!("✅ Mixed path works!");
    }
}
