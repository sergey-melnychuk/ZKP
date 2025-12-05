use ark_bn254::{Bn254, Fq, Fq2, Fr, G1Affine, G2Affine};
use ark_ff::PrimeField;
use ark_groth16::{Groth16, PreparedVerifyingKey, Proof, VerifyingKey};
use num_bigint::BigUint;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
struct ProofJson {
    pi_a: [String; 3],
    pi_b: [[String; 2]; 3],
    pi_c: [String; 3],
}

#[derive(Debug, Deserialize)]
struct VKeyJson {
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

// Conversion helpers
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

fn parse_g1(coords: &[String; 3]) -> G1Affine {
    let x = string_to_fq(&coords[0]);
    let y = string_to_fq(&coords[1]);
    G1Affine::new(x, y)
}

fn parse_g2(coords: &[[String; 2]; 3]) -> G2Affine {
    // snarkjs format: [[x_c1, x_c0], [y_c1, y_c0], [z_c1, z_c0]]
    // We need Fq2::new(c0, c1)
    let x_c0 = string_to_fq(&coords[0][0]);
    let x_c1 = string_to_fq(&coords[0][1]);
    let x = Fq2::new(x_c0, x_c1);
    
    let y_c0 = string_to_fq(&coords[1][0]);
    let y_c1 = string_to_fq(&coords[1][1]);
    let y = Fq2::new(y_c0, y_c1);
    
    G2Affine::new(x, y)
}

// Convert snarkjs JSON to ark-groth16 Proof
fn json_to_proof(proof_json: &ProofJson) -> Proof<Bn254> {
    let a = parse_g1(&proof_json.pi_a);
    let b = parse_g2(&proof_json.pi_b);
    let c = parse_g1(&proof_json.pi_c);
    
    Proof { a, b, c }
}

// Convert snarkjs JSON to ark-groth16 VerifyingKey
fn json_to_vkey(vkey_json: &VKeyJson) -> VerifyingKey<Bn254> {
    let alpha_g1 = parse_g1(&vkey_json.vk_alpha_1);
    let beta_g2 = parse_g2(&vkey_json.vk_beta_2);
    let gamma_g2 = parse_g2(&vkey_json.vk_gamma_2);
    let delta_g2 = parse_g2(&vkey_json.vk_delta_2);
    
    // Parse IC (input commitments) - gamma_abc_g1 вin ark-groth16
    let gamma_abc_g1: Vec<G1Affine> = vkey_json
        .ic
        .iter()
        .map(|coords| parse_g1(coords))
        .collect();
    
    VerifyingKey {
        alpha_g1,
        beta_g2,
        gamma_g2,
        delta_g2,
        gamma_abc_g1,
    }
}

fn main() {
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║   Groth16 Verifier (using ark-groth16 library)       ║");
    println!("╚══════════════════════════════════════════════════════╝\n");
    
    // Load files
    println!("📂 Loading files...");
    let proof_json = fs::read_to_string("../01-01-sudoku/proof.json")
        .expect("Failed to read proof.json");
    let vk_json = fs::read_to_string("../01-01-sudoku/verification_key.json")
        .expect("Failed to read verification_key.json");
    let public_json = fs::read_to_string("../01-01-sudoku/public.json")
        .expect("Failed to read public.json");
    
    println!("✓ Files loaded\n");
    
    // Parse JSON
    println!("📋 Parsing JSON...");
    let proof_data: ProofJson = serde_json::from_str(&proof_json)
        .expect("Failed to parse proof");
    let vkey_data: VKeyJson = serde_json::from_str(&vk_json)
        .expect("Failed to parse verification key");
    let public_signals: PublicSignals = serde_json::from_str(&public_json)
        .expect("Failed to parse public signals");
    
    println!("✓ JSON parsed");
    println!("  Public inputs: {}", public_signals.0.len());
    println!("  Expected: {}\n", vkey_data.n_public);
    
    // Convert to arkworks types
    println!("🔧 Converting to arkworks types...");
    let proof = json_to_proof(&proof_data);
    let vk = json_to_vkey(&vkey_data);
    let public_inputs: Vec<Fr> = public_signals
        .0
        .iter()
        .map(|s| string_to_fr(s))
        .collect();
    
    println!("✓ Conversion complete");
    println!("  Proof points: A, B, C");
    println!("  VKey points: α, β, γ, δ, {} IC points\n", vk.gamma_abc_g1.len());
    
    // Verify using ark-groth16!
    println!("🔍 Verifying proof with ark-groth16...");
    println!("{}", "═".repeat(54));
    
    let start = std::time::Instant::now();
    let pvk = PreparedVerifyingKey::from(vk);
    let result = Groth16::<Bn254>::verify_proof(&pvk, &proof, &public_inputs);
    let duration = start.elapsed();
    
    println!("{}", "═".repeat(54));
    
    match result {
        Ok(true) => {
            println!("\n✅ PROOF VALID!");
            println!("   The sudoku solution is correct!");
        }
        Ok(false) => {
            println!("\n❌ PROOF INVALID!");
            println!("   The proof verification failed!");
        }
        Err(e) => {
            println!("\n❌ VERIFICATION ERROR!");
            println!("   Error: {:?}", e);
        }
    }
    
    println!("\n⏱️  Verification time: {:?}", duration);
    
    // Comparison with manual verification
    println!("{}\n", "─".repeat(54));
    println!("💡 What just happened:");
    println!("   Your manual verifier: ~200 lines of code");
    println!("   ark-groth16::verify(): 1 function call");
    println!("   Both do the exact same thing:");
    println!("   • Parse proof points");
    println!("   • Compute public input commitment");
    println!("   • Verify pairing equation");
    println!("   • Return result");
    println!("{}", "─".repeat(54));
}
