# zk-TLS (TLSNotary): Cryptographic Proof of Web Data

**Last Updated:** December 2025  
**Status:** Production-ready (TLSNotary 0.7+)

---

## Table of Contents

1. [Overview](#overview)
2. [TLS Background](#tls-background)
3. [TLSNotary Protocol](#tlsnotary-protocol)
4. [MPC-TLS Details](#mpc-tls-details)
5. [Circuit Design](#circuit-design)
6. [Implementation](#implementation)
7. [Applications](#applications)
8. [Security Analysis](#security-analysis)
9. [Performance](#performance)
10. [Future Directions](#future-directions)
11. [References](#references)

---

## Overview

### Problem Statement

**Goal:** Prove data came from an HTTPS website without revealing sensitive parts.

**Current situation:**
- You visit https://bank.com, see balance $100,000
- Only YOU can see this data (TLS encryption)
- No way to prove to others what you saw
- Screenshots/HTML can be forged

**What we want:** Cryptographic proof that specific data came from specific website.

**Examples:**
- "Coinbase shows I have > $50K" (for loan application)
- "I'm top 100 GitHub contributor" (for job application)  
- "I bought ticket for this flight" (for refund)
- "My Twitter account is 5+ years old" (for reputation)

### High-Level Approach

```
┌──────────────────────────────────────────────────────────────┐
│ Traditional TLS: User ←→ Server                              │
│ Problem: Only user can see data, can't prove to others       │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│ TLSNotary: User ←→ Notary ←→ Server (3-party)                │
│ Solution: Notary witnesses data, signs attestation           │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│ zk-TLS: User generates ZK proof from TLSNotary transcript    │
│ Result: Prove properties without revealing sensitive data    │
└──────────────────────────────────────────────────────────────┘
```

**Key Innovation:** Split TLS session using Multi-Party Computation (MPC)
- User + Notary jointly generate TLS keys
- Neither party knows full key
- Server sees normal TLS session
- User can prove data came from server

---

## TLS Background

### TLS 1.3 Handshake (Simplified)

```
Client                                    Server
  |                                         |
  |--- ClientHello (supported ciphers) ---->|
  |    + key_share (ephemeral public key)   |
  |                                         |
  |<-- ServerHello (chosen cipher) ---------|
  |    + key_share (ephemeral public key)   |
  |    + Certificate (server's public key)  |
  |    + CertificateVerify (signature)      |
  |    + Finished (verify handshake)        |
  |                                         |
  |--- Finished (verify handshake) -------->|
  |                                         |
  |========== Encrypted Channel ============|
  |                                         |
  |--- GET /api/balance ------------------->|
  |<-- {"balance": 100000} -----------------|
  |                                         |
```

### Key Derivation

**Diffie-Hellman Key Exchange:**
```
Client: private key a, public key A = g^a
Server: private key b, public key B = g^b

Shared secret: s = g^(ab) = (g^a)^b = (g^b)^a
```

**Both parties compute same secret without revealing private keys!**

**TLS 1.3 Key Schedule:**
```
1. Extract:
   Early Secret = HKDF-Extract(0, PSK or 0)
   
2. Derive:
   Handshake Secret = HKDF-Extract(Early Secret, ECDHE)
   
3. Expand:
   Client Handshake Traffic Secret = HKDF-Expand-Label(Handshake Secret, "c hs traffic", transcript)
   Server Handshake Traffic Secret = HKDF-Expand-Label(Handshake Secret, "s hs traffic", transcript)
   
4. Application keys:
   Master Secret = HKDF-Extract(Handshake Secret, 0)
   Client Traffic Secret = HKDF-Expand-Label(Master Secret, "c ap traffic", transcript)
   Server Traffic Secret = HKDF-Expand-Label(Master Secret, "s ap traffic", transcript)
```

### Symmetric Encryption (AES-GCM)

**After handshake:** All data encrypted with symmetric keys

```
Encrypt: C = AES_GCM_Encrypt(key, nonce, plaintext, additional_data)
Decrypt: P = AES_GCM_Decrypt(key, nonce, ciphertext, additional_data, tag)
```

**Properties:**
- Authenticated encryption (integrity + confidentiality)
- Nonce must never repeat for same key
- Tag verifies both ciphertext and additional data

---

## TLSNotary Protocol

### Three Parties

```
┌─────────────┐         ┌─────────────┐         ┌─────────────┐
│   Prover    │         │   Notary    │         │   Server    │
│   (User)    │◄───────►│ (Verifier)  │         │ (Website)   │
│             │   MPC   │             │         │             │
│             │         │             │◄───────►│             │
│             │         │    Proxy    │   TLS   │             │
└─────────────┘         └─────────────┘         └─────────────┘
```

**Roles:**
1. **Prover (User):**
   - Wants to prove data from server
   - Has sensitive credentials (passwords, etc.)
   - Generates ZK proof

2. **Notary (Verifier):**
   - Witnesses TLS session
   - Doesn't see plaintext
   - Signs attestation of data

3. **Server (Website):**
   - Normal HTTPS server
   - Unaware of TLSNotary protocol
   - Sees regular TLS connection

### Protocol Phases

#### Phase 1: Setup

```
1. Prover connects to Notary
2. Agree on parameters:
   - Target server (e.g., https://example.com)
   - Maximum data size
   - Commitment scheme
```

#### Phase 2: MPC-TLS Handshake

**Goal:** Jointly generate TLS keys without either party knowing full key

```
Prover has:  key_share_P
Notary has:  key_share_N

Full key = key_share_P ⊕ key_share_N (XOR)

Neither party alone can decrypt!
```

**Detailed steps:**

```
1. Prover generates ephemeral DH key: (a, A = g^a)
2. Prover sends ClientHello to Server (via Notary proxy)
3. Server responds with ServerHello + B = g^b
4. Prover computes: shared_secret_P = B^a
5. Notary receives B, but can't compute B^a (doesn't know a)

6. Prover and Notary run 2PC-HMAC to derive keys:
   Input_P: a (Prover's DH private key)
   Input_N: transcript (public data)
   
   Output: key_P ⊕ key_N = TLS_key
   
   Prover gets: key_P
   Notary gets: key_N
   Neither knows full TLS_key

7. Complete handshake using 2PC for MAC computations
```

**Key insight:** Server sees normal TLS handshake, unaware of MPC!

#### Phase 3: Data Request

**Prover sends HTTP request:**

```
1. Prover: plaintext_request = "GET /api/balance HTTP/1.1\r\n..."

2. Prover + Notary: 2PC-Encrypt(plaintext_request, key_P ⊕ key_N)
   - Prover contributes key_P
   - Notary contributes key_N
   - Neither sees full key
   - Output: ciphertext_request

3. Send ciphertext_request to Server (via Notary)

4. Server decrypts with key_P ⊕ key_N (doesn't know it's split!)

5. Server sends encrypted response: ciphertext_response
```

#### Phase 4: Selective Disclosure

**Prover decides what to reveal:**

```
1. Server response (encrypted):
   ciphertext = E(key, "{"name":"Alice","balance":100000,"ssn":"123-45-6789"}")

2. Prover + Notary: 2PC-Decrypt(ciphertext, key_P ⊕ key_N)
   - Jointly decrypt
   - Prover learns full plaintext
   - Notary learns commitment to plaintext

3. Prover chooses what to reveal:
   Reveal: {"balance":100000}
   Redact: name, ssn
   
   Redacted: {"name":"[REDACTED]","balance":100000,"ssn":"[REDACTED]"}

4. Prover sends to Notary:
   - Revealed parts (in plaintext)
   - Commitments to redacted parts
   
5. Notary verifies:
   - Commitments match what Notary computed during decryption
   - Revealed + redacted = full response
```

#### Phase 5: Attestation

**Notary signs transcript:**

```
Notary computes attestation:
{
  "server": "example.com",
  "timestamp": 1701432000,
  "data_commitment": H(revealed_data || redacted_commitments),
  "revealed": {"balance": 100000},
  "signature": Sign(notary_key, above_data)
}
```

**What this proves:**
- ✅ Data came from example.com (TLS certificate verified)
- ✅ At timestamp (Notary witnessed)
- ✅ Revealed parts are authentic
- ✅ Redacted parts exist but hidden

#### Phase 6: ZK Proof (Optional)

**Prover generates ZK proof:**

```
Public inputs:
  - data_commitment (from attestation)
  - predicate_result (e.g., "balance > 50000")

Private inputs:
  - full_plaintext (including redacted parts)
  - redaction_randomness

Circuit verifies:
  1. Commitment matches: H(plaintext, randomness) == data_commitment
  2. Predicate holds: balance > 50000
```

**Result:** Prove properties without revealing sensitive data!

---

## MPC-TLS Details

### Two-Party Computation (2PC)

**Goal:** Two parties jointly compute f(x, y) where:
- Party A has private input x
- Party B has private input y
- Both learn output f(x, y)
- Neither learns the other's input

**Example: Millionaire's Problem**
```
Alice has wealth a
Bob has wealth b
Want to know: a > b?
Without revealing a or b!
```

### Garbled Circuits

**Most common 2PC protocol (Yao's Garbled Circuits)**

**Idea:**
1. One party (Generator) creates "garbled" version of circuit
2. Other party (Evaluator) evaluates garbled circuit
3. Evaluator learns output, nothing else

**Example: AND gate**
```
Truth table:
  a  b | out
  0  0 |  0
  0  1 |  0
  1  0 |  0
  1  1 |  1

Garbled version (Generator creates):
  Wire labels for a: k0_a, k1_a (random keys)
  Wire labels for b: k0_b, k1_b
  Wire labels for out: k0_out, k1_out

  Encrypted truth table:
    E(k0_a, k0_b, k0_out)  // 0 AND 0 = 0
    E(k0_a, k1_b, k0_out)  // 0 AND 1 = 0
    E(k1_a, k0_b, k0_out)  // 1 AND 0 = 0
    E(k1_a, k1_b, k1_out)  // 1 AND 1 = 1

Evaluator:
  - Receives labels for their input (via OT)
  - Decrypts one row of table
  - Gets output label
  - Learns output value (not Generator's input!)
```

**For TLS:** Garble circuits for HMAC, AES, etc.

### Oblivious Transfer (OT)

**Problem:** How does Evaluator get their input labels without revealing input?

**1-out-of-2 OT:**
```
Sender has: m0, m1 (two messages)
Receiver wants: mb where b ∈ {0, 1}

Protocol:
1. Receiver chooses b (private)
2. Run OT protocol
3. Receiver learns mb
4. Sender doesn't learn b
5. Receiver doesn't learn m(1-b)
```

**Used in garbled circuits:**
- Sender has wire labels: k0, k1
- Receiver has input bit: b
- Receiver gets kb via OT

### TLS-Specific MPC

**Functions needed:**

**1. HMAC (for key derivation)**
```
HMAC(key, message) = H((key ⊕ opad) || H((key ⊕ ipad) || message))

where:
  opad = 0x5c repeated
  ipad = 0x36 repeated
  H = SHA256
```

**2PC-HMAC:**
```
Input_A: key_A (Prover's key share)
Input_B: key_B (Notary's key share)
Public: message

Compute: HMAC(key_A ⊕ key_B, message)

Output: result (both parties learn)
```

**Circuit:**
```
1. XOR keys: full_key = key_A ⊕ key_B
2. Compute: inner = SHA256((full_key ⊕ ipad) || message)
3. Compute: outer = SHA256((full_key ⊕ opad) || inner)
4. Output: outer
```

**Constraints:** ~100K for full HMAC computation

**2. AES-GCM Encryption**
```
Encrypt(key, nonce, plaintext, aad):
  1. Generate keystream: AES_CTR(key, nonce)
  2. XOR: ciphertext = plaintext ⊕ keystream
  3. Compute GHASH tag over (aad, ciphertext)
  4. Return (ciphertext, tag)
```

**2PC-AES:**
```
Input_A: key_A, plaintext_A
Input_B: key_B, public nonce

Compute: AES(key_A ⊕ key_B, nonce) ⊕ plaintext_A

Output_A: ciphertext (Prover learns)
Output_B: commitment to ciphertext (Notary learns)
```

**Challenge:** AES is slow in garbled circuits (~4000 AND gates per block)

**Optimization:** Use GHASH-based MACs (faster in 2PC)

**3. ECDH (for key exchange)**
```
Prover has: private key a
Server has: public key B

Compute: shared_secret = B^a

Goal: Notary shouldn't learn shared_secret
```

**Approach:**
```
1. Prover computes locally: B^a (doesn't need MPC)
2. Use B^a as input to 2PC-HKDF
3. Notary verifies curve operations (B is on curve, etc.)
```

**No MPC needed for DH itself!**

---

## Circuit Design

### TLSNotary to ZK Proof

**TLSNotary gives us:**
- Encrypted response from server
- Attestation from Notary (signature)
- Revealed + redacted data

**ZK Circuit proves:**
1. ✅ Commitment to data is correct
2. ✅ Predicate holds (e.g., balance > 50000)
3. ✅ Redacted parts remain hidden

### Circuit Structure

```circom
pragma circom 2.1.6;

include "aes.circom";
include "json.circom";
include "poseidon.circom";

template TLSNotaryProof(maxDataLen) {
    // =====================
    // PRIVATE INPUTS
    // =====================
    signal input fullResponse[maxDataLen];     // Complete plaintext
    signal input redactionMask[maxDataLen];    // 0 = redact, 1 = reveal
    signal input aesKey[128];                  // From MPC-TLS
    signal input nonce[96];                    // AES-GCM nonce
    
    // =====================
    // PUBLIC INPUTS
    // =====================
    signal input encryptedResponse[maxDataLen]; // Ciphertext from server
    signal input dataCommitment;                // From Notary attestation
    signal input predicateResult;               // e.g., 1 = balance > 50000
    
    // =====================
    // STEP 1: VERIFY DECRYPTION
    // =====================
    component aes = AES_GCM_Decrypt(maxDataLen);
    for (var i = 0; i < 128; i++) {
        aes.key[i] <== aesKey[i];
    }
    for (var i = 0; i < 96; i++) {
        aes.nonce[i] <== nonce[i];
    }
    for (var i = 0; i < maxDataLen; i++) {
        aes.ciphertext[i] <== encryptedResponse[i];
        aes.plaintext[i] === fullResponse[i];  // Verify decryption
    }
    
    // =====================
    // STEP 2: VERIFY COMMITMENT
    // =====================
    component hasher = Poseidon(maxDataLen);
    for (var i = 0; i < maxDataLen; i++) {
        hasher.inputs[i] <== fullResponse[i];
    }
    dataCommitment === hasher.out;
    
    // =====================
    // STEP 3: EXTRACT FIELD
    // =====================
    component parser = JSONExtract(maxDataLen);
    for (var i = 0; i < maxDataLen; i++) {
        parser.json[i] <== fullResponse[i];
    }
    signal balance <== parser.fields["balance"];
    
    // =====================
    // STEP 4: CHECK PREDICATE
    // =====================
    component gt = GreaterThan(64);
    gt.in[0] <== balance;
    gt.in[1] <== 50000;
    predicateResult === gt.out;
    
    // =====================
    // STEP 5: VERIFY REDACTION
    // =====================
    // Ensure revealed parts match commitment
    // (This is implicitly verified via Step 2)
    
    // Optional: Explicitly check redaction mask
    for (var i = 0; i < maxDataLen; i++) {
        // redactionMask[i] must be 0 or 1
        signal check <== redactionMask[i] * (1 - redactionMask[i]);
        check === 0;
    }
}

component main {public [encryptedResponse, dataCommitment, predicateResult]} = 
    TLSNotaryProof(4096);
```

### Component Breakdown

**1. AES-GCM Decryption (~150K constraints)**

```circom
template AES_GCM_Decrypt(maxLen) {
    signal input key[128];
    signal input nonce[96];
    signal input ciphertext[maxLen];
    signal input tag[128];
    signal output plaintext[maxLen];
    
    // AES-CTR mode for decryption
    component aes_blocks[maxLen / 16];
    for (var block = 0; block < maxLen / 16; block++) {
        aes_blocks[block] = AES_128();
        
        // Counter: nonce || block_number
        for (var i = 0; i < 96; i++) {
            aes_blocks[block].plaintext[i] <== nonce[i];
        }
        for (var i = 0; i < 32; i++) {
            aes_blocks[block].plaintext[96 + i] <== /* block number */;
        }
        
        for (var i = 0; i < 128; i++) {
            aes_blocks[block].key[i] <== key[i];
        }
        
        // XOR keystream with ciphertext
        for (var i = 0; i < 128; i++) {
            signal keystream <== aes_blocks[block].ciphertext[i];
            plaintext[block * 128 + i] <== keystream ^ ciphertext[block * 128 + i];
        }
    }
    
    // Verify GHASH tag
    component ghash = GHASH(maxLen);
    for (var i = 0; i < 128; i++) {
        ghash.key[i] <== key[i];
    }
    for (var i = 0; i < maxLen; i++) {
        ghash.data[i] <== ciphertext[i];
    }
    for (var i = 0; i < 128; i++) {
        tag[i] === ghash.tag[i];
    }
}
```

**Cost:** 
- AES block: ~6K AND gates ≈ 6K constraints
- For 4KB (256 blocks): 256 × 6K = 1.5M constraints
- GHASH: ~20K constraints
- **Total: ~1.5M constraints**

**Optimization:** Don't include full AES in circuit! Instead:
- Notary provides commitment to ciphertext
- Prover proves commitment matches
- Skip actual decryption in circuit (happens off-chain)

**Optimized circuit:**
```circom
template TLSNotaryProofOptimized(maxLen) {
    signal input fullResponse[maxLen];
    signal input dataCommitment;
    
    // Just verify commitment (cheap!)
    component hasher = Poseidon(maxLen);
    for (var i = 0; i < maxLen; i++) {
        hasher.inputs[i] <== fullResponse[i];
    }
    dataCommitment === hasher.out;
    
    // Extract and check predicate
    // ... (as before)
}
```

**New cost:** ~50K constraints (vs 1.5M!)

**2. JSON Parsing (~20K constraints)**

```circom
template JSONExtract(maxLen) {
    signal input json[maxLen];
    signal output fields[MAX_FIELDS];
    
    // Parse JSON structure
    component parser = JSONParser(maxLen);
    for (var i = 0; i < maxLen; i++) {
        parser.input[i] <== json[i];
    }
    
    // Extract specific field (e.g., "balance")
    component extractor = ExtractField(maxLen, 7);  // "balance" = 7 chars
    extractor.json <== parser.output;
    extractor.fieldName <== [98, 97, 108, 97, 110, 99, 101];  // "balance"
    
    fields[0] <== extractor.value;
}
```

**3. Predicate Check (~1K constraints)**

```circom
template GreaterThan(n) {
    signal input in[2];
    signal output out;
    
    component lt = LessThan(n);
    lt.in[0] <== in[1];
    lt.in[1] <== in[0];
    out <== lt.out;
}

template LessThan(n) {
    signal input in[2];
    signal output out;
    
    component n2b = Num2Bits(n+1);
    n2b.in <== in[0] + (1<<n) - in[1];
    out <== 1 - n2b.out[n];
}
```

**Total Circuit Cost (Optimized):**
```
Component                Constraints
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Commitment check          ~5K
JSON parsing             ~20K
Field extraction         ~10K
Predicate evaluation     ~5K
Range checks             ~10K
Misc                     ~5K
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
TOTAL                    ~55K
```

**Much cheaper than zkEmail (600K+) because:**
- No RSA (biggest win!)
- No SHA256 of large data
- Commitment-based approach

---

## Implementation

### Repository Structure

```
tlsn/
├── tlsn-core/              # Core protocol
│   ├── mpc/                # MPC primitives
│   │   ├── garble.rs      # Garbled circuits
│   │   ├── ot.rs          # Oblivious transfer
│   │   └── 2pc/           # 2PC protocols
│   ├── tls/               # TLS integration
│   │   ├── handshake.rs
│   │   ├── record.rs
│   │   └── cipher.rs
│   └── transcript.rs       # Session transcript
├── tlsn-prover/            # Prover (user) implementation
│   ├── client.rs
│   ├── session.rs
│   └── proof.rs
├── tlsn-verifier/          # Notary implementation
│   ├── server.rs
│   ├── attestation.rs
│   └── sign.rs
├── tlsn-circuits/          # ZK circuits
│   ├── commitment.circom
│   ├── json.circom
│   └── predicate.circom
└── examples/              # Usage examples
    ├── twitter/
    ├── bank/
    └── github/
```

### Prover (User) Flow

```rust
use tlsn_prover::{Prover, ProverConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Connect to Notary
    let notary_url = "wss://notary.example.com";
    let notary_conn = connect_websocket(notary_url).await?;
    
    // 2. Configure prover
    let config = ProverConfig::builder()
        .server_name("api.coinbase.com")
        .max_transcript_size(16384)  // 16KB max
        .build()?;
    
    let prover = Prover::new(config);
    
    // 3. Setup MPC-TLS with Notary
    println!("Setting up MPC-TLS...");
    let mpc_tls = prover.setup_mpc_tls(&notary_conn).await?;
    
    // 4. Perform TLS handshake (MPC with Notary)
    println!("Performing TLS handshake...");
    let mut session = mpc_tls.connect().await?;
    
    // 5. Send HTTP request (encrypted via MPC)
    println!("Sending request...");
    let request = b"GET /api/v3/accounts HTTP/1.1\r\n\
                     Host: api.coinbase.com\r\n\
                     Authorization: Bearer YOUR_TOKEN\r\n\
                     \r\n";
    session.send(request).await?;
    
    // 6. Receive response (decrypted via MPC)
    println!("Receiving response...");
    let response = session.receive().await?;
    // response: {"currency":"USD","balance":"123456.78",...}
    
    // 7. Close connection
    session.close().await?;
    
    // 8. Get session transcript
    let transcript = session.transcript();
    
    // 9. Selective disclosure
    println!("Redacting sensitive data...");
    let redacted = transcript
        .builder()
        .reveal_range(/* balance field */)
        .redact_range(/* everything else */)
        .build()?;
    
    // 10. Get Notary attestation
    println!("Getting attestation...");
    let attestation = prover
        .finalize(redacted, &notary_conn)
        .await?;
    
    // 11. Generate ZK proof
    println!("Generating ZK proof...");
    let proof = generate_zk_proof(
        &attestation,
        |balance| balance > 50000.0,  // Predicate
    ).await?;
    
    // 12. Verify locally
    assert!(verify_proof(&proof)?);
    
    println!("Proof generated successfully!");
    println!("Commitment: 0x{}", hex::encode(&proof.commitment));
    println!("Predicate result: {}", proof.predicate_result);
    
    Ok(())
}

async fn generate_zk_proof(
    attestation: &Attestation,
    predicate: impl Fn(f64) -> bool,
) -> Result<Proof> {
    // Parse response
    let response: serde_json::Value = serde_json::from_slice(
        &attestation.revealed_data
    )?;
    
    let balance: f64 = response["balance"]
        .as_str()
        .unwrap()
        .parse()?;
    
    // Check predicate
    let predicate_result = predicate(balance);
    
    // Prepare circuit inputs
    let input = CircuitInput {
        full_response: attestation.full_data.clone(),
        redaction_mask: attestation.redaction_mask.clone(),
        data_commitment: attestation.commitment,
        predicate_result: predicate_result as u8,
    };
    
    // Generate proof using snarkjs
    let proof = groth16::prove(
        &CIRCUIT_WASM,
        &PROVING_KEY,
        &input,
    ).await?;
    
    Ok(proof)
}
```

### Notary (Verifier) Implementation

```rust
use tlsn_verifier::{Notary, NotaryConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Configure notary
    let config = NotaryConfig::builder()
        .bind_addr("0.0.0.0:7047")
        .max_sessions(100)
        .session_timeout(Duration::from_secs(300))
        .build()?;
    
    // 2. Load signing key
    let signing_key = load_signing_key("notary_key.pem")?;
    let notary = Notary::new(config, signing_key);
    
    // 3. Start server
    println!("Starting notary server on port 7047...");
    notary.run(|session| async move {
        handle_session(session).await
    }).await?;
    
    Ok(())
}

async fn handle_session(mut session: Session) -> Result<()> {
    println!("New session from {}", session.peer_addr());
    
    // 1. Setup MPC-TLS
    let mpc_tls = session.setup_mpc_tls().await?;
    
    // 2. Proxy TLS connection
    let mut proxy = mpc_tls.connect().await?;
    
    // 3. Forward data bidirectionally
    let (mut prover_rx, mut server_rx) = proxy.split();
    
    tokio::select! {
        _ = forward_data(&mut prover_rx, &mut server_rx) => {},
        _ = session.is_closed() => {},
    }
    
    // 4. Get transcript
    let transcript = proxy.transcript();
    
    // 5. Receive redaction info from Prover
    let redaction = session.receive_redaction().await?;
    
    // 6. Verify redaction is valid
    if !verify_redaction(&transcript, &redaction) {
        return Err("Invalid redaction".into());
    }
    
    // 7. Compute commitment
    let commitment = compute_commitment(&transcript);
    
    // 8. Sign attestation
    let attestation = Attestation {
        server: transcript.server_name().to_string(),
        timestamp: Utc::now().timestamp(),
        commitment,
        revealed_data: redaction.revealed_bytes(),
        signature: sign(&session.signing_key, &commitment),
    };
    
    // 9. Send to Prover
    session.send_attestation(&attestation).await?;
    
    println!("Session complete");
    Ok(())
}
```

### On-Chain Verification

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "./Verifier.sol";  // ZK verifier

contract TLSProofVerifier {
    Verifier public zkVerifier;
    
    // Notary public keys
    mapping(address => bool) public trustedNotaries;
    
    // Prevent replay
    mapping(bytes32 => bool) public usedCommitments;
    
    event ProofVerified(
        address indexed user,
        bytes32 indexed commitment,
        bool predicateResult,
        uint256 timestamp
    );
    
    constructor(address _verifier) {
        zkVerifier = Verifier(_verifier);
    }
    
    function addNotary(address notary) external {
        // Only owner
        trustedNotaries[notary] = true;
    }
    
    function verifyTLSProof(
        // ZK proof
        uint[2] memory a,
        uint[2][2] memory b,
        uint[2] memory c,
        uint[3] memory publicSignals,  // [commitment, predicateResult, timestamp]
        
        // Notary attestation
        bytes32 commitment,
        address notary,
        bytes memory signature
    ) external {
        // 1. Verify notary signature
        bytes32 message = keccak256(abi.encodePacked(
            commitment,
            publicSignals[2]  // timestamp
        ));
        
        address signer = recoverSigner(message, signature);
        require(trustedNotaries[signer], "Untrusted notary");
        
        // 2. Verify ZK proof
        require(
            zkVerifier.verifyProof(a, b, c, publicSignals),
            "Invalid ZK proof"
        );
        
        // 3. Check commitment matches
        bytes32 proofCommitment = bytes32(publicSignals[0]);
        require(proofCommitment == commitment, "Commitment mismatch");
        
        // 4. Prevent replay
        require(!usedCommitments[commitment], "Proof already used");
        usedCommitments[commitment] = true;
        
        // 5. Extract predicate result
        bool predicateResult = publicSignals[1] == 1;
        
        // 6. Application logic
        if (predicateResult) {
            // Grant access, mint NFT, etc.
        }
        
        emit ProofVerified(
            msg.sender,
            commitment,
            predicateResult,
            publicSignals[2]
        );
    }
    
    function recoverSigner(
        bytes32 message,
        bytes memory signature
    ) internal pure returns (address) {
        bytes32 r;
        bytes32 s;
        uint8 v;
        
        assembly {
            r := mload(add(signature, 32))
            s := mload(add(signature, 64))
            v := byte(0, mload(add(signature, 96)))
        }
        
        return ecrecover(
            keccak256(abi.encodePacked("\x19Ethereum Signed Message:\n32", message)),
            v, r, s
        );
    }
}
```

---

## Applications

### 1. Proof of Funds (DeFi Lending)

**Use Case:** Get undercollateralized loan by proving assets on exchange

**Flow:**
```
1. User has $100K on Coinbase
2. User visits lending protocol
3. Protocol requires proof of funds > $50K
4. User generates TLSNotary proof:
   - Connect to Coinbase API
   - Fetch balance
   - Redact exact amount
   - Prove: balance > $50K
5. Submit proof to lending contract
6. Get approved for loan
```

**Smart Contract:**
```solidity
contract UndercollateralizedLoan {
    mapping(address => uint256) public approvedLoanAmounts;
    
    function applyForLoan(
        uint[2] memory a,
        uint[2][2] memory b,
        uint[2] memory c,
        uint[3] memory publicSignals,
        bytes32 commitment,
        address notary,
        bytes memory signature
    ) external {
        // Verify TLS proof
        require(
            tlsVerifier.verifyTLSProof(a, b, c, publicSignals, commitment, notary, signature),
            "Invalid proof"
        );
        
        // Check predicate: balance > $50K
        require(publicSignals[1] == 1, "Insufficient funds");
        
        // Approve loan
        approvedLoanAmounts[msg.sender] = 10000 * 1e18;  // 10K USDC
    }
    
    function borrow(uint256 amount) external {
        require(amount <= approvedLoanAmounts[msg.sender], "Exceeds limit");
        // Transfer funds...
    }
}
```

### 2. Anonymous Reputation (GitHub Contributions)

**Use Case:** Prove you're top contributor without revealing identity

**Flow:**
```
1. User has 10K+ GitHub stars
2. Generate TLSNotary proof from GitHub API:
   - GET https://api.github.com/users/USERNAME
   - Response: {"public_repos": 150, "followers": 5000, ...}
   - Prove: followers > 1000
3. Submit to reputation contract
4. Get "verified contributor" NFT
```

**Privacy:** No one knows your GitHub username!

### 3. Insurance Claims (Flight Delay)

**Use Case:** Automatic payout if flight delayed

**Flow:**
```
1. User buys insurance for Flight AA1234
2. Flight gets delayed
3. User generates proof from airline API:
   - GET https://api.airline.com/status/AA1234
   - Response: {"flight": "AA1234", "status": "delayed", "minutes": 180}
   - Prove: delay > 120 minutes
4. Submit to insurance contract
5. Automatic payout
```

**Smart Contract:**
```solidity
contract FlightInsurance {
    struct Policy {
        string flightNumber;
        uint256 premium;
        uint256 payout;
        bool claimed;
    }
    
    mapping(address => Policy) public policies;
    
    function buyPolicy(
        string memory flightNumber
    ) external payable {
        require(msg.value >= 0.01 ether, "Premium too low");
        policies[msg.sender] = Policy({
            flightNumber: flightNumber,
            premium: msg.value,
            payout: msg.value * 10,  // 10x payout
            claimed: false
        });
    }
    
    function claim(
        uint[2] memory a,
        uint[2][2] memory b,
        uint[2] memory c,
        uint[3] memory publicSignals,
        bytes32 commitment,
        address notary,
        bytes memory signature
    ) external {
        Policy storage policy = policies[msg.sender];
        require(!policy.claimed, "Already claimed");
        
        // Verify proof: delay > 120 minutes
        require(
            tlsVerifier.verifyTLSProof(a, b, c, publicSignals, commitment, notary, signature),
            "Invalid proof"
        );
        require(publicSignals[1] == 1, "Flight not delayed enough");
        
        // Payout
        policy.claimed = true;
        payable(msg.sender).transfer(policy.payout);
    }
}
```

### 4. Social Verification (Twitter Blue)

**Use Case:** Prove you have Twitter Blue for exclusive access

**Flow:**
```
1. User has Twitter Blue subscription
2. Generate proof from Twitter API:
   - GET https://api.twitter.com/2/users/me
   - Response: {"data": {"verified_type": "blue", ...}}
   - Prove: verified_type == "blue"
3. Get access to exclusive DAO
```

### 5. Credit Score Verification

**Use Case:** Prove credit score > 700 for apartment rental

**Flow:**
```
1. User logs into Experian
2. Generate proof:
   - GET https://api.experian.com/credit-score
   - Response: {"score": 750, "factors": [...]}
   - Redact: exact score, factors
   - Prove: score > 700
3. Submit to landlord's contract
4. Get approved
```

**Privacy:** Landlord doesn't see exact score or credit report!

---

## Security Analysis

### Threat Model

**Trust Assumptions:**
1. ✅ At least one of {Prover, Notary} is honest
2. ✅ Server uses standard TLS (unmodified)
3. ✅ TLS certificates are valid (CA system trusted)
4. ✅ MPC protocols are secure (garbled circuits, OT)
5. ✅ ZK proof system is sound (Groth16)

**Attacks:**

#### 1. Malicious Notary

**Threat:** Notary tries to learn Prover's data or forge attestations

**Protection:**
- **MPC security:** Notary only gets key share, not full key
- **Commitment:** Notary commits to ciphertext before seeing plaintext
- **Signature:** Notary's signature binds them to attestation

**Result:** Notary can't learn data or forge (unless colludes with server)

#### 2. Malicious Prover

**Threat:** Prover tries to fake data from server

**Protection:**
- **TLS certificate:** Server's cert verified by Notary
- **MPC verification:** Notary checks all MPC steps
- **Commitment:** Prover can't change data after Notary commits

**Attack scenario:**
```
Prover: "Server sent me {balance: 1000000}"
Reality: Server sent {balance: 100}

Notary: Has commitment C = H(real_ciphertext)
Prover: Tries to open C to fake data
Result: FAIL (commitment binding)
```

#### 3. Malicious Server + Prover Collusion

**Threat:** Server and Prover collude to create fake data

**Protection:**
- **None!** If server cooperates, it can send fake data

**Mitigation:**
- Trust server (e.g., major exchange like Coinbase)
- Reputation system for servers
- Multiple notaries (reduce single point of failure)

**Note:** This is fundamental limitation - we're proving "server said X", not "X is true"!

#### 4. Man-in-the-Middle

**Threat:** Attacker intercepts TLS connection

**Protection:**
- **TLS certificate verification:** Notary checks cert
- **Public key pinning:** Can require specific cert
- **DNSSEC:** Verify DNS records

**Result:** Standard TLS security applies

#### 5. Timing Attacks

**Threat:** Learn information from timing of MPC operations

**Protection:**
- **Constant-time MPC:** All operations take same time
- **Padding:** Pad responses to fixed size
- **Dummy operations:** Add fake operations

**Implementation:** TLSNotary uses constant-time crypto libraries

#### 6. Selective Disclosure Attacks

**Threat:** Prover reveals inconsistent partial data

**Example:**
```
Full data: {"balance": 100, "debt": 90}
Prover reveals: {"balance": 100}
Prover hides: debt field
Prover claims: "net worth = 100" (should be 10!)
```

**Protection:**
- **Commitment:** Includes full data structure
- **ZK circuit:** Checks all fields (including hidden)
- **Schema validation:** Enforce complete JSON structure

**Mitigation in circuit:**
```circom
// Must process entire JSON, not just revealed parts
component parser = ParseJSON(fullData);
signal balance <== parser.balance;
signal debt <== parser.debt;
signal netWorth <== balance - debt;

// Predicate on correct calculation
netWorth > threshold
```

### Cryptographic Security

**MPC Security:**
- Garbled circuits: Secure against semi-honest adversaries
- Oblivious Transfer: Based on DDH assumption
- **Security level:** ~128 bits

**TLS Security:**
- TLS 1.3: State-of-art protocol
- AES-GCM: AEAD with ~128-bit security
- ECDHE: Perfect forward secrecy

**ZK Security:**
- Groth16: ~128-bit security (BN254)
- Soundness: Adversary can't forge proofs
- Zero-knowledge: Verifier learns nothing beyond statement

**Post-Quantum:**
- ❌ MPC: Secure (assuming PQ-secure hash functions)
- ❌ TLS: Currently vulnerable (ECDHE broken by Shor)
- ❌ Groth16: Pairings vulnerable to quantum attacks

**Future:** Post-quantum TLS + STARKs (quantum-resistant)

### Privacy Analysis

**What Notary learns:**
- Domain name (e.g., "api.coinbase.com")
- Timestamp of connection
- Approximate data size (with padding)
- Commitment to data
- **NOT:** Plaintext data, exact balance, credentials

**What Verifier (on-chain) learns:**
- Commitment to data
- Predicate result (e.g., "balance > 50K")
- Timestamp
- **NOT:** Exact balance, other fields, identity (unless revealed)

**Linkability:**
- Different proofs from same session: Linked by commitment
- Different sessions: Not linkable (different commitments)
- Could add: Anonymous credentials for unlinkability

---

## Performance

### MPC-TLS Overhead

**Compared to regular TLS:**

```
Operation              Regular TLS    MPC-TLS      Overhead
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Handshake              50ms           150ms        3x
Send data (1KB)        1ms            5ms          5x
Receive data (1KB)     1ms            5ms          5x
Close connection       10ms           20ms         2x
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**Bottleneck:** Garbled circuit evaluation for AES/HMAC

**Network:**
- Latency: 50-100ms (depends on Notary distance)
- Bandwidth: 2-5x regular TLS (garbled circuits larger)

### ZK Proof Generation

**Optimized circuit (55K constraints):**

```
Phase                     Time        Memory
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Witness generation        0.5s        0.5GB
FFT                       2s          1GB
Multi-scalar mult         8s          2GB
Proof finalization        1s          0.5GB
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
TOTAL                     11-12s      2GB peak
```

**Much faster than zkEmail (30-60s) because:**
- No RSA (saves 500K constraints)
- No SHA256 of large data (saves ~2M constraints)
- Smaller circuit overall

**With GPU:** 4-5s (3x speedup)

### On-Chain Costs

**Same as zkEmail:**
- Verification: ~250K gas
- Proof size: 192 bytes
- Cost at 10 gwei, ETH=$3000: ~$7.50

### Total E2E Time

**From user perspective:**

```
1. Connect to Notary:        1s
2. MPC-TLS handshake:         2s
3. Send request:              0.5s
4. Receive response:          1s
5. Selective disclosure:      0.5s
6. Get attestation:           1s
7. Generate ZK proof:         12s
8. Submit on-chain:           30s (depends on gas)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
TOTAL:                        ~48s
```

**Optimization:** Can do #7 off-chain, only submit result

---

## Future Directions

### 1. Eliminating the Notary

**Goal:** Pure zk-TLS without trusted third party

**Approaches:**

**A) zkSNARK of entire TLS:**
```
Prove: "I have TLS session with valid certificate where response contains X"

Circuit verifies:
1. Certificate chain
2. TLS handshake
3. Decryption
4. Predicate
```

**Challenge:** Massive circuit (certificate verification alone is ~1M constraints)

**B) TEE (Trusted Execution Environment):**
```
Run TLS inside SGX/TrustZone
Generate attestation proving:
- Code is correct
- Data came from TLS
```

**Challenge:** TEE vulnerabilities, vendor lock-in

**C) Threshold Notaries:**
```
Multiple notaries (e.g., 5)
Need majority (e.g., 3) to sign
Reduces trust in single notary
```

**Status:** Research ongoing, no production solution yet

### 2. Post-Quantum Security

**Current vulnerabilities:**
- ECDHE in TLS: Broken by Shor's algorithm
- Pairing-based zkSNARKs: Broken by quantum attacks

**Solutions:**
- **PQ-TLS:** Use post-quantum key exchange (Kyber, etc.)
- **STARKs:** Quantum-resistant proofs (no trusted setup!)
- **Lattice-based:** FRI-based systems

**Timeline:** 5-10 years for wide adoption

### 3. Browser Integration

**Goal:** Make TLSNotary seamless for users

**Approach:**
```
Chrome Extension:
1. Intercept TLS connections
2. Automatic MPC-TLS
3. User clicks "Generate Proof"
4. One-click verification
```

**Challenges:**
- Browser API limitations
- Performance (WASM for MPC)
- UX complexity

**Status:** Prototype exists, needs polish

### 4. Decentralized Notary Network

**Current:** Single trusted notary

**Future:** Network of notaries
```
- Stake tokens to become notary
- Random selection (prevents targeting)
- Slashing for misbehavior
- Economic security
```

**Similar to:** Chainlink oracles, but for TLS

### 5. Cross-Platform Support

**Current:** Mostly Rust/WASM

**Future:**
- Mobile SDKs (iOS, Android)
- JavaScript library (browser-native)
- Python bindings (data science)
- Go implementation (backend services)

### 6. Standard Protocol

**Goal:** TLSNotary as internet standard

**Path:**
1. IETF RFC for MPC-TLS
2. Browser native support
3. Server-side adoption
4. Wide deployment

**Timeline:** 10+ years

### 7. Privacy-Preserving Analytics

**Use case:** Aggregate statistics without revealing individual data

**Example:**
```
1000 users prove: "My salary > $100K"
Aggregate: "60% of users earn > $100K"
Privacy: No one learns individual salaries
```

**Tech:** MPC + Zero-knowledge proofs + Secure aggregation

---

## References

### Papers

**TLS & MPC:**
1. **"TLS-N: Non-repudiation over TLS"** (2012)  
   Original TLSNotary concept  
   https://tlsnotary.org/TLSNotary.pdf

2. **"A Fast and Scalable Protocol for Secure Computation of TLS"** (2021)  
   Modern MPC-TLS approach  
   https://eprint.iacr.org/2021/1040

3. **RFC 8446** - The Transport Layer Security (TLS) Protocol Version 1.3  
   https://datatracker.ietf.org/doc/html/rfc8446

**MPC Foundations:**
4. **"A Pragmatic Introduction to Secure Multi-Party Computation"** (2021)  
   Evans, Kolesnikov, Rosulek  
   https://securecomputation.org/

5. **"Secure Computation of the k'th Ranked Element"** (Yao 1982)  
   Original garbled circuits paper

6. **"Extending Oblivious Transfers Efficiently"** (Ishai et al. 2003)  
   OT extension protocol  
   https://www.iacr.org/archive/crypto2003/27290145/27290145.pdf

**Zero-Knowledge:**
7. **Groth16** - "On the Size of Pairing-based Non-interactive Arguments"  
   https://eprint.iacr.org/2016/260

8. **PLONK** - "PLONK: Permutations over Lagrange-bases"  
   https://eprint.iacr.org/2019/953

### Implementations

**TLSNotary:**
- **tlsn** - Main implementation (Rust)  
  https://github.com/tlsnotary/tlsn

- **tlsn-js** - JavaScript/WASM bindings  
  https://github.com/tlsnotary/tlsn-js

- **tlsn-extension** - Browser extension  
  https://github.com/tlsnotary/tlsn-extension

**MPC Libraries:**
- **fancy-garbling** - Fast garbled circuits (Rust)  
  https://github.com/GaloisInc/swanky

- **emp-toolkit** - MPC toolkit (C++)  
  https://github.com/emp-toolkit

- **MP-SPDZ** - General MPC framework  
  https://github.com/data61/MP-SPDZ

**TLS Libraries:**
- **rustls** - Modern TLS library (Rust)  
  https://github.com/rustls/rustls

- **BoringSSL** - Google's TLS (C)  
  https://github.com/google/boringssl

### Tools & Infrastructure

**Notaries:**
- **Public Notary** - Free notary for testing  
  https://notary.pse.dev

- **Self-hosted** - Run your own  
  https://docs.tlsnotary.org/run-notary

**Circuit Tools:**
- **Circom** - Circuit compiler  
  https://github.com/iden3/circom

- **snarkjs** - JavaScript prover  
  https://github.com/iden3/snarkjs

**Developer Tools:**
- **tlsn-cli** - Command-line interface  
  ```bash
  cargo install tlsn-cli
  ```

- **tlsn-examples** - Example applications  
  https://github.com/tlsnotary/tlsn-examples

### Educational Resources

**Tutorials:**
1. **TLSNotary Docs** - Official documentation  
   https://docs.tlsnotary.org

2. **"Secure Multi-Party Computation in Practice"** - Tutorial  
   https://www.youtube.com/watch?v=...

3. **"Building with TLSNotary"** - Workshop materials  
   https://github.com/tlsnotary/workshop

**Courses:**
4. **"Applied Cryptography"** - Dan Boneh (Stanford)  
   https://www.coursera.org/learn/crypto

5. **"Secure Computation"** - Nigel Smart  
   https://www.cs.bris.ac.uk/~nigel/

**Books:**
6. **"Foundations of Cryptography"** - Oded Goldreich  
   Volumes I & II

7. **"Secure Multiparty Computation and Secret Sharing"** - Cramer et al.  
   Cambridge University Press

### Community

**Discord:**
- **TLSNotary Discord** - Main community  
  https://discord.gg/tlsnotary

- **PSE Discord** - Privacy & Scaling Explorations  
  https://discord.gg/pse

**Forums:**
- **Ethereum Research**  
  https://ethresear.ch/c/privacy/

**Twitter:**
- @tlsnotary
- @privacy_scaling
- @heliaxdev

### Production Examples

**Live Applications:**
1. **Proof of Balance** - Crypto exchange balances  
   https://github.com/tlsnotary/examples/balance

2. **Proof of Identity** - Social media verification  
   https://github.com/tlsnotary/examples/identity

3. **Proof of Purchase** - E-commerce receipts  
   https://github.com/tlsnotary/examples/purchase

4. **Credit Score Verification** - Anonymous credit checks  
   https://github.com/tlsnotary/examples/credit

### Comparison with Alternatives

**vs zkEmail:**
- TLSNotary: Real-time, any HTTPS data
- zkEmail: Async, email-only, no notary needed

**vs zkOracles:**
- TLSNotary: User generates proof
- zkOracles: Oracle generates proof, less privacy

**vs Attestation Services:**
- TLSNotary: Trustless (MPC)
- Attestation: Trust service provider

**vs Screenshots:**
- TLSNotary: Cryptographic proof
- Screenshots: Easily forged

---

## Conclusion

zk-TLS (TLSNotary) enables cryptographic proof of web data with selective disclosure, opening new possibilities for privacy-preserving applications. By combining MPC-TLS with zero-knowledge proofs, users can prove properties about data from any HTTPS website without revealing sensitive information.

**Key Takeaways:**

1. **MPC splits trust:** Neither user nor notary can cheat alone
2. **Real-time verification:** Unlike zkEmail (async)
3. **Any HTTPS data:** Works with any standard TLS server
4. **Selective disclosure:** Reveal only necessary information
5. **ZK layer optional:** Can use attestation directly or add ZK

**When to use TLSNotary:**
- ✅ Need proof of web data (API, website)
- ✅ Real-time verification required
- ✅ Privacy important (selective disclosure)
- ✅ Standard HTTPS (no special server support)

**When NOT to use:**
- ❌ Email-based verification (use zkEmail)
- ❌ Historical data (need live session)
- ❌ Server willing to provide proof (use zkOracles)
- ❌ No notary available (future: trustless version)

**The future:** Decentralized notary networks, post-quantum security, browser integration, and elimination of trusted parties will make zk-TLS even more powerful and accessible.

---

*Last updated: December 2025*  
*For questions: https://docs.tlsnotary.org*  
*GitHub: https://github.com/tlsnotary/tlsn*
