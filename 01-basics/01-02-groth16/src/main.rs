use ark_bn254::{Bn254, Fq, Fq2, Fr, G1Affine, G2Affine};
use ark_ec::pairing::Pairing;
use ark_ec::AffineRepr;
use ark_ff::{PrimeField, Zero};
use num_bigint::BigUint;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
struct Proof {
    pi_a: [String; 3],
    pi_b: [[String; 2]; 3],
    pi_c: [String; 3],
}

#[derive(Debug, Deserialize)]
struct VerificationKey {
    #[serde(rename = "nPublic")]
    n_public: usize,
    vk_alpha_1: [String; 3],
    vk_beta_2: [[String; 2]; 3],
    vk_gamma_2: [[String; 2]; 3],
    vk_delta_2: [[String; 2]; 3],
    #[serde(rename = "IC")]
    ic: Vec<[String; 3]>,
}

#[derive(Debug, Deserialize)]
struct PublicSignals(Vec<String>);

// Convert decimal number into a field element
fn string_to_fq(s: &str) -> Fq {
    let bigint = BigUint::parse_bytes(s.as_bytes(), 10).expect("Invalid number");
    let bytes = bigint.to_bytes_be();
    Fq::from_be_bytes_mod_order(&bytes)
}

fn string_to_fr(s: &str) -> Fr {
    let bigint = BigUint::parse_bytes(s.as_bytes(), 10).expect("Invalid number");
    let bytes = bigint.to_bytes_be();
    Fr::from_be_bytes_mod_order(&bytes)
}

// Parse G1 point from snarkjs format [x, y, z]
fn parse_g1(coords: &[String; 3]) -> G1Affine {
    println!("Parsing G1 point:");
    println!("  x: {}", &coords[0]);
    println!("  y: {}", &coords[1]);
    println!("  z: {}", &coords[2]);

    let x = string_to_fq(&coords[0]);
    let y = string_to_fq(&coords[1]);

    println!("  Parsed x: {:?}", x);
    println!("  Parsed y: {:?}", y);

    let point = G1Affine::new(x, y);
    println!("  On curve: {}", point.is_on_curve());

    point
}

// Parse G2 point from snarkjs format [[x1, x0], [y1, y0], [z1, z0]]
fn parse_g2(coords: &[[String; 2]; 3]) -> G2Affine {
    println!("Parsing G2 point:");
    println!("  x: [{}, {}]", &coords[0][0], &coords[0][1]);
    println!("  y: [{}, {}]", &coords[1][0], &coords[1][1]);
    println!("  z: [{}, {}]", &coords[2][0], &coords[2][1]);
    
    let x_c0 = string_to_fq(&coords[0][0]);
    let x_c1 = string_to_fq(&coords[0][1]);
    let x = Fq2::new(x_c0, x_c1);
    
    let y_c0 = string_to_fq(&coords[1][0]);
    let y_c1 = string_to_fq(&coords[1][1]);
    let y = Fq2::new(y_c0, y_c1);
    
    println!("  Parsed x: {:?}", x);
    println!("  Parsed y: {:?}", y);
    
    let point = G2Affine::new(x, y);
    println!("  On curve: {}", point.is_on_curve());
    
    point
}

fn verify_groth16(proof: &Proof, vk: &VerificationKey, public_inputs: &[Fr]) -> bool {
    println!("Parsing proof points...");

    // 1. Parse proof
    let proof_a = parse_g1(&proof.pi_a);
    let proof_b = parse_g2(&proof.pi_b);
    let proof_c = parse_g1(&proof.pi_c);

    println!("Proof A: {:?}", proof_a);
    println!("Proof B: {:?}", proof_b);
    println!("Proof C: {:?}", proof_c);

    // 2. Parse verification key
    let vk_alpha = parse_g1(&vk.vk_alpha_1);
    let vk_beta = parse_g2(&vk.vk_beta_2);
    let vk_gamma = parse_g2(&vk.vk_gamma_2);
    let vk_delta = parse_g2(&vk.vk_delta_2);

    println!("\nVerification key parsed");

    // 3. Compute linear combination of public inputs
    println!("\nComputing public input commitment...");

    let mut vk_x = parse_g1(&vk.ic[0]);
    println!("IC[0]: {:?}", vk_x);

    for (i, input) in public_inputs.iter().enumerate() {
        let ic_point = parse_g1(&vk.ic[i + 1]);
        let scaled = ic_point.mul_bigint(input.into_bigint());
        vk_x = (vk_x + scaled).into();
        println!("Added IC[{}] * input[{}]", i + 1, i);
    }

    println!("Public input commitment: {:?}", vk_x);

    // 4. Verify pairing equation
    println!("\nComputing pairings...");

    let pairing1 = Bn254::pairing(proof_a, proof_b);
    println!("e(A, B) computed");

    let pairing2 = Bn254::pairing(-vk_alpha, vk_beta);
    println!("e(-α, β) computed");

    let pairing3 = Bn254::pairing(-vk_x, vk_gamma);
    println!("e(-vk_x, γ) computed");

    let pairing4 = Bn254::pairing(-proof_c, vk_delta);
    println!("e(-C, δ) computed");

    let result = pairing1 + pairing2 + pairing3 + pairing4;

    println!("\nFinal pairing result: {:?}", result);

    result.is_zero()
}

fn main() {
    println!("Groth16 Verifier\n");

    // Load files
    println!("Loading files...");
    let proof_json =
        fs::read_to_string("../01-01-sudoku/proof.json").expect("Failed to read proof.json");
    let vk_json = fs::read_to_string("../01-01-sudoku/verification_key.json")
        .expect("Failed to read verification_key.json");
    let public_json =
        fs::read_to_string("../01-01-sudoku/public.json").expect("Failed to read public.json");

    // Parse JSON
    println!("Parsing JSON...");
    let proof: Proof = serde_json::from_str(&proof_json).expect("Failed to parse proof");
    let vk: VerificationKey =
        serde_json::from_str(&vk_json).expect("Failed to parse verification key");
    let public_signals: PublicSignals =
        serde_json::from_str(&public_json).expect("Failed to parse public signals");

    println!("Number of public inputs: {}", public_signals.0.len());
    println!("VK expects {} public inputs\n", vk.n_public);

    // Convert public inputs to field elements
    let public_inputs: Vec<Fr> = public_signals.0.iter().map(|s| string_to_fr(s)).collect();

    // Verify!
    println!("{}", "=".repeat(50));
    let is_valid = verify_groth16(&proof, &vk, &public_inputs);
    println!("{}", "=".repeat(50));

    println!(
        "\n🔍 Proof verification result: {}",
        if is_valid { "✅ VALID" } else { "❌ INVALID" }
    );
}
