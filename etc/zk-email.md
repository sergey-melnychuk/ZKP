# zkEmail: Cryptographic Proof of Email Contents

**Last Updated:** December 2025  
**Status:** Production-ready, multiple implementations

---

## Table of Contents

1. [Overview](#overview)
2. [DKIM Background](#dkim-background)
3. [Architecture](#architecture)
4. [Circuit Design](#circuit-design)
5. [Implementation Details](#implementation-details)
6. [Applications](#applications)
7. [Security Analysis](#security-analysis)
8. [Performance](#performance)
9. [Future Directions](#future-directions)
10. [References](#references)

---

## Overview

### Problem Statement

**Goal:** Prove properties about an email without revealing its contents.

**Examples:**
- "I received an email from google.com" (without showing what it says)
- "This invoice shows amount > $10,000" (without revealing exact amount)
- "I own user@university.edu" (without revealing which university)
- "I got a password reset link" (for account recovery)

**Key Insight:** Email has built-in authentication (DKIM signatures) that we can verify in zero-knowledge!

### High-Level Approach

```
┌─────────────────────────────────────────────────────────────┐
│ 1. Email arrives with DKIM signature                        │
│    (Server's RSA signature over email headers + body)       │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 2. User has: Email + DKIM public key (from DNS)             │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 3. ZK Circuit verifies:                                     │
│    - RSA signature is valid                                 │
│    - Email is from claimed domain                           │
│    - Extract specific fields (amount, date, etc)            │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 4. Output: ZK proof + public signals                        │
│    - Proof: "Email is authentic and contains X"             │
│    - Public: domain hash, extracted field                   │
│    - Private: Full email contents, signature                │
└─────────────────────────────────────────────────────────────┘
```

---

## DKIM Background

### What is DKIM?

**DomainKeys Identified Mail (RFC 6376)** - Email authentication protocol using public-key cryptography.

**Purpose:** Prevent email spoofing and tampering.

**How it works:**

1. **Sending server** signs email with private key
2. **Public key** published in DNS (TXT record)
3. **Receiving server** verifies signature using public key from DNS
4. **Verification proves:** Email came from claimed domain and wasn't modified

### DKIM Signature Structure

**Example email with DKIM:**

```
DKIM-Signature: v=1; a=rsa-sha256; c=relaxed/relaxed;
        d=google.com; s=20230601;
        h=to:subject:message-id:date:from:mime-version:from:to:cc:subject
         :date:message-id:reply-to;
        bh=uXRJvWPfS8CnOT5hPPz7uf8tM9w6e6p8pRXlHgL1234=;
        b=XYZ123...ABC789==
From: noreply@google.com
To: user@example.com
Subject: Verify your account
Date: Wed, 01 Dec 2025 10:30:00 +0000
Message-ID: <abc123@google.com>

Please verify your account by clicking:
https://accounts.google.com/verify?token=xyz
```

**DKIM Header Fields:**

```
v=1                  → DKIM version
a=rsa-sha256         → Algorithm (RSA signature, SHA256 hash)
c=relaxed/relaxed    → Canonicalization (how to normalize text)
d=google.com         → Signing domain
s=20230601           → Selector (which key to use)
h=from:to:subject... → Which headers are signed
bh=...               → Body hash (base64 of SHA256(body))
b=...                → Signature (RSA signature of headers)
```

**Verification Process:**

```python
# 1. Extract signed headers
headers = extract_headers(email, dkim.h)  # from, to, subject, date

# 2. Canonicalize (normalize whitespace, line endings)
canonical_headers = canonicalize(headers, dkim.c)
canonical_body = canonicalize(body, dkim.c)

# 3. Hash body
body_hash = base64(sha256(canonical_body))
assert body_hash == dkim.bh

# 4. Create signing string
signing_string = canonical_headers + "\r\n" + dkim_header_without_signature

# 5. Hash signing string
message_hash = sha256(signing_string)

# 6. Fetch public key from DNS
pubkey = dns_lookup(f"{dkim.s}._domainkey.{dkim.d}")  # 20230601._domainkey.google.com

# 7. Verify RSA signature
assert rsa_verify(dkim.b, message_hash, pubkey)
```

### DNS Public Key Format

**Query:**
```bash
dig TXT 20230601._domainkey.google.com
```

**Response:**
```
v=DKIM1; k=rsa; p=MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...
```

**Fields:**
- `v=DKIM1` - Version
- `k=rsa` - Key type
- `p=...` - Public key (base64 encoded)

**Public key structure (RSA):**
```
N = modulus (2048 bits typical)
e = exponent (usually 65537)
```

### DKIM Security Properties

**What DKIM guarantees:**
1. ✅ Email came from domain in `d=` field
2. ✅ Signed headers weren't modified
3. ✅ Body wasn't modified (via body hash)
4. ✅ Signature was made by holder of private key

**What DKIM doesn't guarantee:**
1. ❌ Email reached intended recipient (no encryption)
2. ❌ Email wasn't forwarded/relayed
3. ❌ Sender identity (only domain, not specific person)
4. ❌ Non-repudiation (sender can revoke keys)

**For zkEmail:** DKIM is perfect! It's a cryptographic proof that email is authentic.

---

## Architecture

### System Components

```
┌──────────────────────────────────────────────────────────────────┐
│                    zkEmail Full Stack                             │
└──────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ 1. Frontend (User Interface)                                    │
│    - Email upload                                                │
│    - Field selection (what to prove)                            │
│    - Proof generation trigger                                    │
└─────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│ 2. Parser (Email Processing)                                    │
│    - Extract DKIM signature                                     │
│    - Parse headers (From, To, Subject, Date)                   │
│    - Extract body                                               │
│    - Canonicalize text                                          │
│    - Fetch DKIM public key from DNS                            │
└─────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│ 3. Witness Generator                                            │
│    - Convert email to circuit inputs                            │
│    - Pad to fixed size (circuit constraint)                     │
│    - Convert RSA signature/key to bigint chunks                 │
│    - Generate regex matches                                     │
└─────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│ 4. ZK Circuit (Circom)                                          │
│    ┌──────────────────────────────────────────────────────┐     │
│    │ a. RSA Signature Verification                        │     │
│    │    - BigInt exponentiation: sig^e mod N              │     │
│    │    - Compare with hash                               │     │
│    │    ~500K constraints                                 │     │
│    └──────────────────────────────────────────────────────┘     │
│    ┌──────────────────────────────────────────────────────┐     │
│    │ b. SHA256 Hashing                                    │     │
│    │    - Hash headers + body                             │     │
│    │    ~25K constraints per hash                         │     │
│    └──────────────────────────────────────────────────────┘     │
│    ┌──────────────────────────────────────────────────────┐     │
│    │ c. Domain Extraction                                 │     │
│    │    - Parse "From:" header                            │     │
│    │    - Extract domain                                  │     │
│    │    ~10K constraints                                  │     │
│    └──────────────────────────────────────────────────────┘     │
│    ┌──────────────────────────────────────────────────────┐     │
│    │ d. Field Extraction (Regex)                          │     │
│    │    - Pattern matching in body                        │     │
│    │    - Extract amounts, dates, etc                     │     │
│    │    ~50K-100K constraints                             │     │
│    └──────────────────────────────────────────────────────┘     │
│                                                                 │
│    Total: ~600K - 1M constraints                                │
└─────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│ 5. Proof System (Groth16)                                       │
│    - Generate witness                                           │
│    - Compute proof (30-60s)                                     │
│    - Output: 192-byte proof                                     │
└─────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│ 6. Smart Contract (On-chain Verification)                       │
│    - Verifier contract (auto-generated)                         │
│    - Verify proof (250K gas)                                    │
│    - Check public signals                                       │
│    - Execute logic (mint NFT, allow action, etc)                │
└─────────────────────────────────────────────────────────────────┘
```

### Data Flow

**Input (Private):**
```javascript
{
  emailHeader: "From: google@google.com\r\nTo: user@...",  // Raw header
  emailBody: "Please verify...",                            // Raw body
  signature: [chunk0, chunk1, ...],                         // RSA sig (2048 bits)
  modulus: [chunk0, chunk1, ...],                           // RSA N (2048 bits)
}
```

**Output (Public):**
```javascript
{
  domainHash: poseidon("google.com"),      // Proves email from google.com
  nullifier: poseidon(emailHeader),        // Prevents double-use
  extractedField: 10000,                    // E.g., invoice amount
  timestamp: 1701432000,                    // Email date
}
```

**Proof:**
```javascript
{
  pi_a: [x, y],        // Proof point A (G1)
  pi_b: [[x1, x2], [y1, y2]],  // Proof point B (G2)
  pi_c: [x, y],        // Proof point C (G1)
  protocol: "groth16", // 192 bytes total
}
```

---

## Circuit Design

### Overall Structure

```circom
pragma circom 2.1.6;

include "rsa.circom";
include "sha256.circom";
include "extract.circom";
include "regex.circom";

template EmailVerify(
    maxHeaderLen,    // e.g., 1024 bytes
    maxBodyLen,      // e.g., 4096 bytes
    nBits,           // RSA key size, e.g., 2048
    k                // Number of chunks for bigint, e.g., 64 for 2048-bit
) {
    // =====================
    // PRIVATE INPUTS
    // =====================
    signal input emailHeader[maxHeaderLen];
    signal input emailBody[maxBodyLen];
    signal input rsaSignature[k];
    signal input rsaModulus[k];
    
    // =====================
    // PUBLIC INPUTS
    // =====================
    signal output domainHash;
    signal output nullifier;
    signal output extractedAmount;
    
    // =====================
    // COMPONENT 1: SHA256
    // =====================
    // Hash the email (headers + body)
    component sha = SHA256Bytes(maxHeaderLen + maxBodyLen);
    for (var i = 0; i < maxHeaderLen; i++) {
        sha.in[i] <== emailHeader[i];
    }
    for (var i = 0; i < maxBodyLen; i++) {
        sha.in[maxHeaderLen + i] <== emailBody[i];
    }
    
    // =====================
    // COMPONENT 2: RSA VERIFY
    // =====================
    component rsa = RSAVerify65537(nBits, k);
    for (var i = 0; i < k; i++) {
        rsa.signature[i] <== rsaSignature[i];
        rsa.modulus[i] <== rsaModulus[i];
    }
    // Connect hash to RSA verification
    for (var i = 0; i < 256; i++) {
        rsa.hashed[i] <== sha.out[i];
    }
    
    // =====================
    // COMPONENT 3: EXTRACT DOMAIN
    // =====================
    component extractor = ExtractDomain(maxHeaderLen);
    for (var i = 0; i < maxHeaderLen; i++) {
        extractor.header[i] <== emailHeader[i];
    }
    domainHash <== extractor.domainHash;
    
    // =====================
    // COMPONENT 4: EXTRACT FIELD
    // =====================
    component amountExtractor = ExtractAmount(maxBodyLen);
    for (var i = 0; i < maxBodyLen; i++) {
        amountExtractor.body[i] <== emailBody[i];
    }
    extractedAmount <== amountExtractor.amount;
    
    // =====================
    // COMPONENT 5: NULLIFIER
    // =====================
    component nullifierHasher = Poseidon(maxHeaderLen);
    for (var i = 0; i < maxHeaderLen; i++) {
        nullifierHasher.inputs[i] <== emailHeader[i];
    }
    nullifier <== nullifierHasher.out;
}

component main {public [domainHash, extractedAmount]} = EmailVerify(1024, 4096, 2048, 64);
```

### Component 1: RSA Verification

**Challenge:** RSA with 2048-bit keys in arithmetic circuits.

**RSA Signature Verification:**
```
Given:
  signature S (2048 bits)
  public key (e, N) where e = 65537, N = 2048-bit modulus
  message hash H (256 bits, padded to 2048 bits)

Verify:
  S^e mod N == H (with PKCS#1 v1.5 padding)
```

**Circuit Implementation:**

```circom
// RSA verification for e = 65537
template RSAVerify65537(nBits, k) {
    signal input signature[k];   // S in chunks
    signal input modulus[k];     // N in chunks
    signal input hashed[256];    // H (SHA256 output)
    
    // Convert hash to padded format (PKCS#1 v1.5)
    component padder = PKCS1v15Pad(nBits);
    for (var i = 0; i < 256; i++) {
        padder.hash[i] <== hashed[i];
    }
    signal paddedHash[k];
    for (var i = 0; i < k; i++) {
        paddedHash[i] <== padder.out[i];
    }
    
    // Compute S^65537 mod N
    component exp = BigIntModExp(nBits, k, 65537);
    for (var i = 0; i < k; i++) {
        exp.base[i] <== signature[i];
        exp.modulus[i] <== modulus[i];
    }
    
    // Check S^65537 mod N == paddedHash
    for (var i = 0; i < k; i++) {
        exp.out[i] === paddedHash[i];
    }
}
```

**BigInt Modular Exponentiation:**

```circom
// Compute base^exp mod modulus
// For exp = 65537 = 2^16 + 1, we can optimize:
// base^65537 = base^(2^16) * base
template BigIntModExp(nBits, k, exp) {
    signal input base[k];
    signal input modulus[k];
    signal output out[k];
    
    // exp = 65537 = 10000000000000001 in binary
    // We need 16 squarings + 1 multiplication
    
    signal temp[17][k];  // Intermediate results
    
    // temp[0] = base
    for (var i = 0; i < k; i++) {
        temp[0][i] <== base[i];
    }
    
    // 16 squarings
    component squares[16];
    for (var round = 0; round < 16; round++) {
        squares[round] = BigIntModMul(nBits, k);
        for (var i = 0; i < k; i++) {
            squares[round].a[i] <== temp[round][i];
            squares[round].b[i] <== temp[round][i];  // Square
            squares[round].modulus[i] <== modulus[i];
            temp[round + 1][i] <== squares[round].out[i];
        }
    }
    
    // Final multiplication: temp[16] * base
    component final = BigIntModMul(nBits, k);
    for (var i = 0; i < k; i++) {
        final.a[i] <== temp[16][i];
        final.b[i] <== base[i];
        final.modulus[i] <== modulus[i];
        out[i] <== final.out[i];
    }
}
```

**Cost Breakdown:**
- 16 modular squarings: ~25K constraints each = 400K
- 1 modular multiplication: ~25K constraints
- PKCS padding: ~10K constraints
- **Total: ~500K constraints for RSA-2048**

**Why so expensive?**
1. 2048-bit numbers don't fit in native field elements (254 bits for BN254)
2. Must split into 64 × 32-bit chunks
3. Each multiplication requires carries, range checks
4. 16 rounds of squaring

### Component 2: SHA256

**SHA256 in circuits:**

```circom
template SHA256Bytes(maxLen) {
    signal input in[maxLen];
    signal output out[256];
    
    // Pad message to multiple of 512 bits
    signal padded[paddedLen];  // paddedLen = ceil(maxLen / 64) * 64
    component padder = SHA256Pad(maxLen);
    for (var i = 0; i < maxLen; i++) {
        padder.in[i] <== in[i];
    }
    for (var i = 0; i < paddedLen; i++) {
        padded[i] <== padder.out[i];
    }
    
    // Process each 512-bit block
    var numBlocks = paddedLen / 64;
    component blocks[numBlocks];
    signal hash[numBlocks + 1][8][32];  // Hash state after each block
    
    // Initial hash value (SHA256 constants)
    hash[0][0] <== [/* H0 bits */];
    // ... H1 through H7
    
    for (var block = 0; block < numBlocks; block++) {
        blocks[block] = SHA256Compression();
        
        // Input previous hash state
        for (var i = 0; i < 8; i++) {
            for (var j = 0; j < 32; j++) {
                blocks[block].hashIn[i][j] <== hash[block][i][j];
            }
        }
        
        // Input message block (512 bits = 64 bytes)
        for (var i = 0; i < 64; i++) {
            for (var j = 0; j < 8; j++) {
                blocks[block].message[i][j] <== /* extract bit j from byte i */;
            }
        }
        
        // Get output hash state
        for (var i = 0; i < 8; i++) {
            for (var j = 0; j < 32; j++) {
                hash[block + 1][i][j] <== blocks[block].hashOut[i][j];
            }
        }
    }
    
    // Final hash is last state
    for (var i = 0; i < 256; i++) {
        out[i] <== hash[numBlocks][i / 32][i % 32];
    }
}
```

**SHA256 Compression Function:**

```circom
template SHA256Compression() {
    signal input hashIn[8][32];     // Previous hash (a-h)
    signal input message[64][8];    // Message block (512 bits)
    signal output hashOut[8][32];   // New hash
    
    // Message schedule: expand 512 bits to 2048 bits (64 words)
    signal W[64][32];
    for (var i = 0; i < 16; i++) {
        for (var j = 0; j < 32; j++) {
            W[i][j] <== /* convert 8 bytes to 32-bit word */;
        }
    }
    
    // Expand remaining 48 words
    component schedule[48];
    for (var i = 16; i < 64; i++) {
        schedule[i - 16] = SHA256Schedule();
        // W[i] = σ1(W[i-2]) + W[i-7] + σ0(W[i-15]) + W[i-16]
        // ... (omitted for brevity)
    }
    
    // 64 rounds of compression
    signal state[65][8][32];  // Hash state after each round
    for (var i = 0; i < 8; i++) {
        for (var j = 0; j < 32; j++) {
            state[0][i][j] <== hashIn[i][j];
        }
    }
    
    component rounds[64];
    for (var round = 0; round < 64; round++) {
        rounds[round] = SHA256Round();
        // ... complex operations ...
        // T1 = h + Σ1(e) + Ch(e,f,g) + K[round] + W[round]
        // T2 = Σ0(a) + Maj(a,b,c)
        // Update state
    }
    
    // Add to initial hash
    component adders[8];
    for (var i = 0; i < 8; i++) {
        adders[i] = BinAdd32();
        for (var j = 0; j < 32; j++) {
            adders[i].a[j] <== hashIn[i][j];
            adders[i].b[j] <== state[64][i][j];
            hashOut[i][j] <== adders[i].out[j];
        }
    }
}
```

**Cost per block (512 bits):**
- 64 rounds × ~400 constraints = 25,600 constraints
- Message schedule: ~5,000 constraints
- **Total: ~30K constraints per 512-bit block**

**For typical email (4KB body):**
- 4096 bytes = 32,768 bits ≈ 64 blocks
- 64 blocks × 30K = **~2M constraints just for SHA256!**

**Optimization:** Only hash necessary parts, use smaller body limits.

### Component 3: Domain Extraction

**Goal:** Extract "google.com" from "From: user@google.com"

```circom
template ExtractDomain(maxHeaderLen) {
    signal input header[maxHeaderLen];
    signal output domainHash;
    
    // Find "From:" field
    component finder = FindSubstring(maxHeaderLen, 5);  // "From:"
    for (var i = 0; i < maxHeaderLen; i++) {
        finder.text[i] <== header[i];
    }
    finder.pattern <== [70, 114, 111, 109, 58];  // "From:" in ASCII
    
    signal fromIndex <== finder.index;
    
    // Find '@' after "From:"
    component atFinder = FindCharAfter(maxHeaderLen);
    for (var i = 0; i < maxHeaderLen; i++) {
        atFinder.text[i] <== header[i];
    }
    atFinder.start <== fromIndex;
    atFinder.char <== 64;  // '@'
    
    signal atIndex <== atFinder.index;
    
    // Extract domain (characters after '@' until whitespace)
    signal domain[256];  // Max domain length
    component extractor = ExtractUntilWhitespace(maxHeaderLen, 256);
    for (var i = 0; i < maxHeaderLen; i++) {
        extractor.text[i] <== header[i];
    }
    extractor.start <== atIndex + 1;  // After '@'
    for (var i = 0; i < 256; i++) {
        domain[i] <== extractor.out[i];
    }
    
    // Hash domain
    component hasher = Poseidon(256);
    for (var i = 0; i < 256; i++) {
        hasher.inputs[i] <== domain[i];
    }
    domainHash <== hasher.out;
}
```

**Cost:** ~10K-20K constraints (depending on max lengths)

### Component 4: Field Extraction (Regex)

**Example:** Extract amount from "Invoice amount: $1234"

**Approach 1: DFA (Deterministic Finite Automaton)**

```circom
template ExtractAmount(maxBodyLen) {
    signal input body[maxBodyLen];
    signal output amount;
    
    // State machine for pattern: "amount: $" followed by digits
    signal states[maxBodyLen + 1];  // Current state after each character
    states[0] <== 0;  // Initial state
    
    component transitions[maxBodyLen];
    for (var i = 0; i < maxBodyLen; i++) {
        transitions[i] = AmountDFAStep();
        transitions[i].currentState <== states[i];
        transitions[i].char <== body[i];
        states[i + 1] <== transitions[i].nextState;
    }
    
    // Extract digits when in "reading amount" state
    signal digits[maxBodyLen];
    signal inAmountState[maxBodyLen];
    
    for (var i = 0; i < maxBodyLen; i++) {
        // Check if state is in "reading amount" state (e.g., state 10-19)
        component checker = IsInRange();
        checker.value <== states[i + 1];
        checker.min <== 10;
        checker.max <== 19;
        inAmountState[i] <== checker.out;
        
        // If in amount state and char is digit, extract
        component isDigit = IsDigit();
        isDigit.char <== body[i];
        
        digits[i] <== inAmountState[i] * isDigit.out * (body[i] - 48);  // '0' = 48
    }
    
    // Convert digits to number
    component converter = DigitsToNumber(maxBodyLen);
    for (var i = 0; i < maxBodyLen; i++) {
        converter.digits[i] <== digits[i];
        converter.valid[i] <== inAmountState[i];
    }
    amount <== converter.number;
}

template AmountDFAStep() {
    signal input currentState;
    signal input char;
    signal output nextState;
    
    // State transitions:
    // 0: initial
    // 1: saw 'a'
    // 2: saw 'am'
    // 3: saw 'amo'
    // 4: saw 'amou'
    // 5: saw 'amoun'
    // 6: saw 'amount'
    // 7: saw 'amount:'
    // 8: saw 'amount: '
    // 9: saw 'amount: $'
    // 10-19: reading digits
    
    // This is complex - need to implement all transitions
    // Omitted for brevity, but each transition adds constraints
}
```

**Cost:** ~50K-100K constraints (depends on pattern complexity and text length)

**Approach 2: Heuristic Matching (Cheaper)**

```circom
template SimpleAmountExtractor(maxBodyLen) {
    signal input body[maxBodyLen];
    signal output amount;
    
    // Look for '$' character
    component dollarFinder = FindChar(maxBodyLen);
    dollarFinder.char <== 36;  // '$'
    for (var i = 0; i < maxBodyLen; i++) {
        dollarFinder.text[i] <== body[i];
    }
    signal dollarIndex <== dollarFinder.index;
    
    // Extract next N digits after '$'
    signal digits[10];  // Support up to 10-digit amounts
    for (var i = 0; i < 10; i++) {
        signal charAfterDollar <== body[dollarIndex + 1 + i];
        
        component isDigit = IsDigit();
        isDigit.char <== charAfterDollar;
        
        digits[i] <== isDigit.out * (charAfterDollar - 48);
    }
    
    // Convert to number
    signal powers[10];
    powers[0] <== 1;
    for (var i = 1; i < 10; i++) {
        powers[i] <== powers[i - 1] * 10;
    }
    
    signal sum <== 0;
    for (var i = 0; i < 10; i++) {
        sum <== sum + digits[9 - i] * powers[i];
    }
    amount <== sum;
}
```

**Cost:** ~5K-10K constraints (much cheaper!)

---

## Implementation Details

### Repository Structure

```
zk-email/
├── packages/
│   ├── circuits/          # Circom circuits
│   │   ├── email-verifier.circom
│   │   ├── rsa.circom
│   │   ├── sha256.circom
│   │   ├── regex.circom
│   │   └── utils.circom
│   ├── helpers/           # JS utilities
│   │   ├── dkim.ts       # DKIM parsing
│   │   ├── generate-input.ts
│   │   └── relayer.ts
│   ├── contracts/         # Solidity contracts
│   │   ├── EmailVerifier.sol
│   │   ├── DKIMRegistry.sol
│   │   └── Verifier.sol  # Generated by snarkjs
│   └── app/              # Frontend
│       ├── upload.tsx
│       ├── prove.tsx
│       └── verify.tsx
├── zkeys/                # Proving keys
│   ├── email.zkey
│   └── email.vkey.json
└── examples/             # Use case demos
    ├── proof-of-twitter/
    ├── nft-mint/
    └── recovery/
```

### Setup Process

**1. Circuit Compilation**

```bash
# Install dependencies
npm install -g circom snarkjs

# Compile circuit
circom circuits/email-verifier.circom \
    --r1cs --wasm --sym \
    --O1  # Optimization level

# Output:
# - email-verifier.r1cs (constraint system)
# - email-verifier.wasm (witness generator)
# - email-verifier.sym  (debug symbols)
```

**2. Trusted Setup (Powers of Tau)**

```bash
# Phase 1: Universal setup (one-time, can be reused)
snarkjs powersoftau new bn128 20 pot20_0000.ptau
snarkjs powersoftau contribute pot20_0000.ptau pot20_0001.ptau \
    --name="First contribution" -v

# Need log2(constraints) < 20
# For 1M constraints, need 2^20 = 1,048,576
# So k=20 is sufficient

# Phase 2: Circuit-specific setup
snarkjs powersoftau prepare phase2 pot20_0001.ptau pot20_final.ptau
snarkjs groth16 setup email-verifier.r1cs pot20_final.ptau email_0000.zkey

# Contributions for circuit
snarkjs zkey contribute email_0000.zkey email_0001.zkey \
    --name="Circuit contribution" -v

# Export verification key
snarkjs zkey export verificationkey email_0001.zkey vkey.json

# Generate Solidity verifier
snarkjs zkey export solidityverifier email_0001.zkey Verifier.sol
```

**Setup time:** 1-2 hours (for 1M constraint circuit)

**3. DKIM Key Registry**

Smart contract to track DKIM public keys (they rotate!):

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract DKIMRegistry {
    // domain => selector => pubkey hash => is valid
    mapping(bytes32 => mapping(bytes32 => mapping(bytes32 => bool))) public dkimKeys;
    
    // domain => selector => revoked timestamp
    mapping(bytes32 => mapping(bytes32 => uint256)) public revokedAt;
    
    address public owner;
    
    event DKIMPublicKeyAdded(
        bytes32 indexed domainHash,
        bytes32 indexed selectorHash,
        bytes32 pubkeyHash
    );
    
    event DKIMPublicKeyRevoked(
        bytes32 indexed domainHash,
        bytes32 indexed selectorHash
    );
    
    constructor() {
        owner = msg.sender;
    }
    
    function addDKIMKey(
        bytes32 domainHash,
        bytes32 selectorHash,
        bytes32 pubkeyHash
    ) external {
        require(msg.sender == owner, "Only owner");
        dkimKeys[domainHash][selectorHash][pubkeyHash] = true;
        emit DKIMPublicKeyAdded(domainHash, selectorHash, pubkeyHash);
    }
    
    function revokeDKIMKey(
        bytes32 domainHash,
        bytes32 selectorHash
    ) external {
        require(msg.sender == owner, "Only owner");
        revokedAt[domainHash][selectorHash] = block.timestamp;
        emit DKIMPublicKeyRevoked(domainHash, selectorHash);
    }
    
    function isDKIMKeyValid(
        bytes32 domainHash,
        bytes32 selectorHash,
        bytes32 pubkeyHash,
        uint256 emailTimestamp
    ) external view returns (bool) {
        // Key must be in registry
        if (!dkimKeys[domainHash][selectorHash][pubkeyHash]) {
            return false;
        }
        
        // Key must not be revoked before email timestamp
        uint256 revoked = revokedAt[domainHash][selectorHash];
        if (revoked != 0 && emailTimestamp >= revoked) {
            return false;
        }
        
        return true;
    }
}
```

**Why needed?** DKIM keys rotate! A proof might be generated after key rotation.

### Proof Generation

**Full flow:**

```typescript
import { buildPoseidon } from 'circomlibjs';
import * as snarkjs from 'snarkjs';
import { parseEmail, extractDKIM, getDKIMPublicKey } from '@zk-email/helpers';

async function generateEmailProof(rawEmail: string) {
    // 1. Parse email
    const parsed = parseEmail(rawEmail);
    const { header, body, dkim } = parsed;
    
    // 2. Extract DKIM signature
    const {
        signature,      // RSA signature
        domain,         // e.g., "google.com"
        selector,       // e.g., "20230601"
        bodyHash,       // bh= field
        signedHeaders   // h= field
    } = extractDKIM(dkim);
    
    // 3. Fetch DKIM public key from DNS
    const pubkey = await getDKIMPublicKey(domain, selector);
    // pubkey = { n: bigint, e: bigint }
    
    // 4. Convert to circuit inputs
    const headerBytes = Array.from(new TextEncoder().encode(header));
    const bodyBytes = Array.from(new TextEncoder().encode(body));
    
    // Pad to circuit size
    const headerPadded = padToLength(headerBytes, 1024);
    const bodyPadded = padToLength(bodyBytes, 4096);
    
    // Convert RSA signature and modulus to chunks
    const signatureChunks = bigIntToChunks(signature, 64, 32);
    const modulusChunks = bigIntToChunks(pubkey.n, 64, 32);
    
    // 5. Generate witness
    const input = {
        emailHeader: headerPadded,
        emailBody: bodyPadded,
        rsaSignature: signatureChunks,
        rsaModulus: modulusChunks,
    };
    
    // 6. Calculate witness
    const { witness } = await snarkjs.wtns.calculate(
        input,
        'email-verifier.wasm',
        'email.wtns'
    );
    
    // 7. Generate proof (this takes 30-60s)
    console.log('Generating proof... (this may take a minute)');
    const { proof, publicSignals } = await snarkjs.groth16.prove(
        'email.zkey',
        'email.wtns'
    );
    
    // 8. Format for Solidity
    const proofFormatted = {
        pi_a: [proof.pi_a[0], proof.pi_a[1]],
        pi_b: [[proof.pi_b[0][1], proof.pi_b[0][0]], [proof.pi_b[1][1], proof.pi_b[1][0]]],
        pi_c: [proof.pi_c[0], proof.pi_c[1]],
    };
    
    return {
        proof: proofFormatted,
        publicSignals: publicSignals,
        domain: domain,
        timestamp: parsed.date.getTime() / 1000,
    };
}

// Helper functions
function padToLength(arr: number[], len: number): number[] {
    if (arr.length >= len) return arr.slice(0, len);
    return [...arr, ...Array(len - arr.length).fill(0)];
}

function bigIntToChunks(n: bigint, numChunks: number, bitsPerChunk: number): string[] {
    const chunks = [];
    const mask = (1n << BigInt(bitsPerChunk)) - 1n;
    for (let i = 0; i < numChunks; i++) {
        chunks.push(((n >> BigInt(i * bitsPerChunk)) & mask).toString());
    }
    return chunks;
}
```

### Verification (On-chain)

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "./Verifier.sol";  // Generated by snarkjs
import "./DKIMRegistry.sol";

contract EmailProofVerifier {
    Verifier public verifier;
    DKIMRegistry public dkimRegistry;
    
    // Prevent proof replay
    mapping(bytes32 => bool) public usedNullifiers;
    
    event ProofVerified(
        address indexed user,
        bytes32 indexed domainHash,
        bytes32 nullifier,
        uint256 timestamp
    );
    
    constructor(address _verifier, address _registry) {
        verifier = Verifier(_verifier);
        dkimRegistry = DKIMRegistry(_registry);
    }
    
    function verifyEmailProof(
        uint[2] memory a,
        uint[2][2] memory b,
        uint[2] memory c,
        uint[4] memory publicSignals,  // [domainHash, nullifier, extractedField, timestamp]
        bytes32 domainHash,
        bytes32 selectorHash,
        bytes32 pubkeyHash
    ) external {
        // 1. Verify proof
        require(
            verifier.verifyProof(a, b, c, publicSignals),
            "Invalid proof"
        );
        
        // 2. Extract public signals
        bytes32 domainFromProof = bytes32(publicSignals[0]);
        bytes32 nullifier = bytes32(publicSignals[1]);
        uint256 extractedField = publicSignals[2];
        uint256 timestamp = publicSignals[3];
        
        // 3. Check domain matches
        require(domainFromProof == domainHash, "Domain mismatch");
        
        // 4. Check nullifier not used (prevent replay)
        require(!usedNullifiers[nullifier], "Proof already used");
        usedNullifiers[nullifier] = true;
        
        // 5. Verify DKIM key is valid
        require(
            dkimRegistry.isDKIMKeyValid(
                domainHash,
                selectorHash,
                pubkeyHash,
                timestamp
            ),
            "Invalid or revoked DKIM key"
        );
        
        // 6. Execute application logic
        // e.g., mint NFT, allow access, etc.
        
        emit ProofVerified(msg.sender, domainHash, nullifier, timestamp);
    }
}
```

---

## Applications

### 1. Email-Based Account Recovery

**Use Case:** Recover wallet if you lose seed phrase, using email backup.

**Setup:**
```solidity
contract WalletWithEmailRecovery {
    address public owner;
    bytes32 public recoveryEmailHash;  // Hash of recovery email address
    
    function setupRecovery(bytes32 _emailHash) external {
        require(msg.sender == owner);
        recoveryEmailHash = _emailHash;
    }
    
    function initiateRecovery(
        uint[2] memory a,
        uint[2] memory b,
        uint[2] memory c,
        uint[4] memory publicSignals,
        address newOwner
    ) external {
        // Verify proof of email ownership
        bytes32 emailHashFromProof = bytes32(publicSignals[0]);
        require(emailHashFromProof == recoveryEmailHash, "Wrong email");
        
        // Verify proof
        require(emailVerifier.verifyProof(a, b, c, publicSignals), "Invalid proof");
        
        // Time delay for security
        recoveryRequests[newOwner] = block.timestamp + 3 days;
    }
    
    function executeRecovery(address newOwner) external {
        require(block.timestamp >= recoveryRequests[newOwner], "Time lock");
        owner = newOwner;
    }
}
```

**User Flow:**
1. User sets up recovery: "My recovery email is user@gmail.com"
2. User loses wallet access
3. User requests recovery email from service: "Click here to recover wallet"
4. User receives email with unique token
5. User generates ZK proof: "I received email from recovery@service.com with token X"
6. Submit proof → wallet ownership transfers after time delay

**Security:**
- Time delay prevents instant theft
- Original owner can cancel recovery
- Email proves intent (user requested it)

### 2. Proof of Twitter Blue (Anonymous)

**Use Case:** Prove you have Twitter Blue subscription without revealing your handle.

**Email:** Twitter sends "Your Twitter Blue subscription is active"

**Proof:**
```typescript
const proof = generateProof({
    email: rawEmail,
    publicSignals: [
        hash("twitter.com"),                    // Domain
        hash("Twitter Blue subscription"),      // Keyword found
        timestamp,                               // Email date
    ]
});
```

**Smart Contract:**
```solidity
contract TwitterBlueNFT {
    mapping(bytes32 => bool) public claimed;
    
    function claimNFT(
        uint[2] memory a,
        uint[2] memory b,
        uint[2] memory c,
        uint[4] memory publicSignals
    ) external {
        // Verify proof
        require(verifier.verifyProof(a, b, c, publicSignals));
        
        bytes32 nullifier = bytes32(publicSignals[1]);
        require(!claimed[nullifier], "Already claimed");
        
        // Mint NFT
        _mint(msg.sender, tokenId++);
        claimed[nullifier] = true;
    }
}
```

**Privacy:** No one knows your Twitter handle, just that you have Blue.

### 3. Proof of Invoice Payment

**Use Case:** Prove you paid $X without revealing exact amount or invoice details.

**Email from Stripe:** "Payment of $10,543.21 received"

**Circuit extracts:** amount = 10543

**Proof:**
```typescript
const proof = generateProof({
    email: rawEmail,
    predicate: amount > 10000,  // Prove amount > threshold
    publicSignals: [
        hash("stripe.com"),
        hash("greater_than_10000"),
        timestamp
    ]
});
```

**Use Case:** Apply for business loan, prove revenue > $100K without revealing exact revenue.

### 4. Proof of University Email (.edu)

**Use Case:** Get student discount without revealing which university.

**Email:** Verification email from university.edu

**Proof extracts:** Domain ends with ".edu"

```circom
template IsEduDomain(maxLen) {
    signal input domain[maxLen];
    signal output isEdu;
    
    // Check last 4 characters are ".edu"
    signal dot <== domain[maxLen - 4];    // '.'
    signal e <== domain[maxLen - 3];      // 'e'
    signal d <== domain[maxLen - 2];      // 'd'
    signal u <== domain[maxLen - 1];      // 'u'
    
    component check = ANDn(4);
    check.in[0] <== (dot === 46);   // ASCII '.'
    check.in[1] <== (e === 101);    // ASCII 'e'
    check.in[2] <== (d === 100);    // ASCII 'd'
    check.in[3] <== (u === 117);    // ASCII 'u'
    
    isEdu <== check.out;
}
```

**Use:** Get student discount on software, prove student status anonymously.

### 5. ZK KYC (Know Your Customer)

**Use Case:** Prove you passed KYC without revealing personal information.

**Email from KYC provider:** "Your identity verification is complete"

**Proof:**
- Domain: kyc-provider.com
- Contains: "verification complete"
- User address linked

**Smart Contract:**
```solidity
contract ZK_KYC {
    mapping(address => bool) public verified;
    
    function submitKYC(
        uint[2] memory a,
        uint[2] memory b,
        uint[2] memory c,
        uint[4] memory publicSignals
    ) external {
        require(verifier.verifyProof(a, b, c, publicSignals));
        
        // Mark address as KYC verified
        verified[msg.sender] = true;
    }
    
    function requireKYC(address user) external view {
        require(verified[user], "Not KYC verified");
    }
}
```

**Use:** Access regulated DeFi protocols without centralized KYC database.

---

## Security Analysis

### Threat Model

**Assumptions:**
1. ✅ DKIM signatures are unforgeable (RSA-2048 secure)
2. ✅ DNS is trusted (or use DNSSEC)
3. ✅ Email provider doesn't collude with user
4. ✅ ZK proof system is sound (Groth16 security)
5. ✅ Trusted setup was performed honestly (at least 1 honest participant)

**Attacks to Consider:**

#### 1. DKIM Key Compromise

**Threat:** Attacker gets private DKIM key → can sign fake emails

**Mitigation:**
- DKIM keys rotate regularly (6-12 months)
- Track key revocations in DKIMRegistry
- Proofs include email timestamp
- Can invalidate old proofs if key compromised

#### 2. Email Forwarding / Modification

**Threat:** Email forwarded → new DKIM signature by forwarder

**Not an issue:** Circuit verifies DKIM signature of original sender, not forwarder.

**Example:**
```
Original: From: google@google.com, DKIM: d=google.com
Forwarded: From: google@google.com, DKIM: d=forwarder.com

Circuit checks: d=google.com ✓
```

#### 3. Timestamp Manipulation

**Threat:** Use old email to generate proof after key rotation

**Mitigation:**
- Check email timestamp against key validity period
- DKIMRegistry tracks revocation dates
- Reject proofs with timestamps after key revocation

#### 4. Proof Replay

**Threat:** Reuse same proof multiple times

**Mitigation:**
- Nullifier prevents double-use
- Nullifier = hash(email header) → unique per email
- Contract tracks used nullifiers

```solidity
mapping(bytes32 => bool) public usedNullifiers;

function verify(proof, publicSignals) external {
    bytes32 nullifier = publicSignals[1];
    require(!usedNullifiers[nullifier], "Proof already used");
    usedNullifiers[nullifier] = true;
    // ...
}
```

#### 5. Circuit Under-Constraint

**Threat:** Circuit doesn't properly constrain inputs → can forge proofs

**Example vulnerability:**
```circom
// BAD: Using <-- instead of <==
signal output result;
result <-- someComputation();  // Assignment without constraint!
```

**Attacker can set `result` to any value!**

**Mitigation:**
- Formal verification of circuits
- Extensive testing (fuzz testing)
- Audit by ZK experts
- Use well-tested libraries (circomlib)

#### 6. Malleability

**Threat:** Modify email in ways that don't break DKIM but change meaning

**DKIM signs:**
- Selected headers (h= field)
- Body (via body hash)

**Attack:** Add unsigned headers
```
From: evil@evil.com
DKIM-Signature: d=google.com; h=from:to; ...
From: google@google.com
To: user@example.com
```

Only second "From:" is signed!

**Mitigation:**
- Circuit must parse carefully
- Check "From:" header is in signed headers list (h= field)
- Use strictest DKIM parsing

### Cryptographic Security

**RSA-2048:**
- Classical security: ~112 bits
- Quantum security: 0 bits (Shor's algorithm)
- Current recommendation: 2048 bits minimum

**SHA-256:**
- Classical collision resistance: 128 bits
- Quantum collision resistance: ~85 bits (Grover's)
- Preimage resistance: 256 bits classical, 128 bits quantum

**Groth16:**
- Security based on:
  - q-Strong Diffie-Hellman (q-SDH)
  - q-Power Knowledge of Exponent (q-PKE)
- Security level: ~128 bits (BN254 curve)
- Trusted setup required

**Post-Quantum:**
- RSA and elliptic curves vulnerable to quantum computers
- Need post-quantum signatures (e.g., CRYSTALS-Dilithium)
- DKIM doesn't support post-quantum yet

### Privacy Analysis

**What's revealed:**
- Domain hash (e.g., hash("google.com"))
- Extracted fields (e.g., amount > 10000)
- Timestamp (email date)

**What's hidden:**
- Full email contents
- Exact email address (unless explicitly revealed)
- Other fields in email
- RSA signature (it's the witness)

**Linkability:**
- Different proofs from same email: Linked by nullifier
- Different proofs from different emails: Not linkable (random nullifiers)
- Could add unlinkability via group signatures if needed

---

## Performance

### Constraint Count Breakdown

```
Component                 Constraints    Percentage
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
RSA-2048 Verification      500,000       62.5%
SHA256 (headers)            30,000        3.75%
SHA256 (body, 4KB)         240,000       30%
Domain Extraction           15,000        1.875%
Field Extraction            10,000        1.25%
Misc (nullifier, etc)        5,000        0.625%
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
TOTAL                      800,000       100%
```

**Bottleneck:** RSA verification (62.5% of circuit)

### Proof Generation Time

**Hardware:** 8-core CPU, 16GB RAM

```
Phase                     Time        Memory
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Witness generation        2-3s        2GB
FFT (setup)               5-8s        4GB
Multi-scalar mult         20-30s      8GB
Proof finalization        3-5s        2GB
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
TOTAL                     30-45s      8GB peak
```

**With GPU acceleration:** 10-15s (3x speedup)

### Proof Size

**Groth16:** 192 bytes (constant, regardless of circuit size)
- Point A: 64 bytes (2 field elements)
- Point B: 128 bytes (2 × 2 field elements, G2 point)  
- Point C: 64 bytes (2 field elements)

**Public signals:** 32 bytes × 4 = 128 bytes
- Domain hash: 32 bytes
- Nullifier: 32 bytes
- Extracted field: 32 bytes
- Timestamp: 32 bytes

**Total:** 320 bytes (proof + public inputs)

### On-Chain Verification

**Gas cost:** ~250,000 gas

**Breakdown:**
- Pairing precompile: ~150,000 gas
- Field arithmetic: ~50,000 gas
- Contract logic: ~50,000 gas

**Cost at 10 gwei, ETH = $3000:**
- 250,000 × 10 gwei = 0.0025 ETH = $7.50 per verification

**Batching:** Verify 10 proofs for ~500K gas (5x cheaper per proof)

### Optimizations

**1. Reduce Body Size**

Most emails have large bodies, but we only need small parts:

```circom
// Instead of maxBodyLen = 16384 (16KB)
// Use maxBodyLen = 4096 (4KB) or less

// SHA256 constraints: 25K per 512-bit block
// 16KB = 256 blocks = 6.4M constraints
// 4KB = 64 blocks = 1.6M constraints
// Savings: 4.8M constraints (75%!)
```

**2. Use Poseidon Instead of SHA256**

If DKIM supported Poseidon (it doesn't):
- SHA256: 25K constraints per hash
- Poseidon: 150 constraints per hash
- Speedup: ~160x!

**Unfortunately:** Stuck with SHA256 for DKIM compatibility.

**3. Reduce RSA Key Size**

Some domains use RSA-1024 (deprecated):
- RSA-2048: 500K constraints
- RSA-1024: 150K constraints
- Savings: 350K constraints

**Issue:** RSA-1024 is insecure (factored in 2020s), not recommended.

**4. Use EdDSA for DKIM**

If DKIM adopted EdDSA signatures (e.g., Ed25519):
- EdDSA verification: ~2K constraints (250x cheaper than RSA!)
- Much smaller keys (32 bytes vs 256 bytes)

**Status:** Experimental DKIM extensions exist, not widely adopted.

**5. Optimize Regex**

For simple patterns, avoid full DFA:

```circom
// Instead of general regex engine:
template SimpleAmountExtractor() {
    // Find '$', read digits
    // ~5K constraints vs ~50K for DFA
}
```

**Trade-off:** Less flexible, but much cheaper.

**6. Recursive SNARKs**

Generate proof of RSA verification separately, verify that proof in main circuit:

```
Main Circuit:
  - Verify RSA proof (5K constraints vs 500K!)
  - SHA256
  - Extraction
  
RSA Circuit (separate):
  - RSA verification (500K constraints)
  - Generate proof
```

**Total:** Still 500K constraints, but can parallelize or precompute.

---

## Future Directions

### 1. Post-Quantum DKIM

**Problem:** RSA broken by quantum computers (Shor's algorithm)

**Solution:** Post-quantum signatures
- CRYSTALS-Dilithium (NIST standard)
- Falcon
- SPHINCS+

**Challenge:** PQC signatures in ZK circuits
- Dilithium: Lattice-based, ~10K constraints (vs 500K for RSA!)
- Much smaller signatures (~2KB vs 256 bytes)

**Status:** Research stage, no DKIM support yet

### 2. Account Abstraction Integration

**ERC-4337 + zkEmail:**

```solidity
contract ZKEmailSessionKey {
    function validateUserOp(
        UserOperation calldata userOp,
        bytes32 userOpHash,
        uint256 missingAccountFunds
    ) external returns (uint256 validationData) {
        // Instead of ECDSA signature, verify zkEmail proof
        (uint[2] memory a, uint[2] memory b, uint[2] memory c, uint[4] memory signals) =
            abi.decode(userOp.signature, (uint[2], uint[2], uint[2], uint[4]));
        
        require(emailVerifier.verifyProof(a, b, c, signals), "Invalid proof");
        
        // Check email contains valid command
        bytes32 command = bytes32(signals[2]);
        require(isValidCommand(command), "Invalid command");
        
        return 0;  // Valid
    }
}
```

**Use case:** Execute transactions by sending emails!

### 3. Privacy-Preserving Regex

**Current:** Regex pattern is public (part of circuit)

**Future:** Hide regex pattern using techniques like:
- Homomorphic encryption
- Function secret sharing
- Garbled circuits

**Use case:** Company wants to verify employee emails without revealing search pattern.

### 4. Decentralized DKIM Key Registry

**Current:** Centralized DKIMRegistry contract (owner adds keys)

**Future:** Decentralized oracle network
- Chainlink-style oracles fetch DKIM keys from DNS
- Multiple oracles reach consensus
- Automatic key rotation tracking

### 5. Cross-Chain Proofs

**Goal:** Verify zkEmail proof on any chain

**Approach:**
- Generate proof once
- Verify on chain A
- Bridge verification result to chain B, C, D...
- Or: Verify proof directly on each chain (gas cost)

**Use case:** Unified identity across all chains.

### 6. Browser Extension

**Current:** Manual email upload, complex setup

**Future:** Browser extension
1. Intercepts emails in Gmail/Outlook
2. Extracts DKIM automatically
3. Generates proof in background
4. One-click verification

**UX:** Seamless, no technical knowledge required.

### 7. Email Streaming / Selective Disclosure

**Current:** Must reveal entire email to circuit

**Future:** Commit-and-prove scheme
- Commit to email: C = hash(email)
- Prove properties: "C commits to email where field X = Y"
- Never reveal full email, even to circuit

**Tech:** Polynomial commitments, KZG, etc.

---

## References

### Papers

**DKIM:**
1. **RFC 6376** - DomainKeys Identified Mail (DKIM) Signatures  
   https://datatracker.ietf.org/doc/html/rfc6376

2. **RFC 6377** - DomainKeys Identified Mail (DKIM) and Mailing Lists  
   https://datatracker.ietf.org/doc/html/rfc6377

**Zero-Knowledge Proofs:**
3. **Groth16** - "On the Size of Pairing-based Non-interactive Arguments" (2016)  
   Jens Groth  
   https://eprint.iacr.org/2016/260

4. **PLONK** - "PLONK: Permutations over Lagrange-bases for Oecumenical Noninteractive arguments of Knowledge" (2019)  
   Gabizon, Williamson, Ciobotaru  
   https://eprint.iacr.org/2019/953

**RSA in Circuits:**
5. **"Efficient RSA Key Generation and Threshold Paillier in the Two-Party Setting"** (2020)  
   Relevant for understanding RSA in MPC/ZK  
   https://eprint.iacr.org/2020/374

**zkEmail Specific:**
6. **"ZK Email: Email-based Anonymous Credentials"** (2023)  
   Original zkEmail whitepaper  
   https://prove.email/blog/zkemail

7. **"Practical Email Authentication with Zero-Knowledge Proofs"** (2023)  
   Aayush Gupta et al.  
   (Check zkEmail blog for latest papers)

### Implementations

**Primary:**
- **zk-email/zk-email-verify** - Main implementation  
  https://github.com/zk-email/zk-email-verify

- **zk-email/relayer** - Backend relayer for proof generation  
  https://github.com/zk-email/relayer

**Related:**
- **circomlib** - Standard Circom libraries (SHA256, etc.)  
  https://github.com/iden3/circomlib

- **snarkjs** - JavaScript ZK proof toolkit  
  https://github.com/iden3/snarkjs

- **circom** - Circuit compiler  
  https://github.com/iden3/circom

### Tools & Libraries

**DKIM:**
- **node-dkim** - DKIM verification in Node.js  
  https://www.npmjs.com/package/dkim-verify

- **mailparser** - Email parsing library  
  https://www.npmjs.com/package/mailparser

**ZK:**
- **arkworks** - Rust ZK library  
  https://github.com/arkworks-rs

- **gnark** - Go ZK library  
  https://github.com/ConsenSys/gnark

**BigInt Arithmetic:**
- **circom-ecdsa** - ECDSA in Circom (shows bigint techniques)  
  https://github.com/0xPARC/circom-ecdsa

- **circom-pairing** - Pairing-based crypto in Circom  
  https://github.com/yi-sun/circom-pairing

### Educational Resources

**Tutorials:**
1. **0xPARC Learning Resources**  
   https://learn.0xparc.org/

2. **ZK Whiteboard Sessions** (YouTube)  
   https://www.youtube.com/c/ZeroKnowledge

3. **"Why and How zk-SNARKs Work"** - Maksym Petkus  
   https://arxiv.org/abs/1906.07221

**Courses:**
4. **"Zero Knowledge Proofs MOOC"** - Dan Boneh (Stanford)  
   https://zk-learning.org/

5. **"Foundations of Probabilistic Proofs"** - Guy Rothblum  
   Coursera / edX

**Blogs:**
6. **Vitalik Buterin's ZK posts**  
   https://vitalik.ca/general/2021/01/26/snarks.html

7. **a16z Crypto Canon**  
   https://a16z.com/2022/04/15/crypto-canon-zero-knowledge-proofs/

### Community

**Discord:**
- **ZK Email Discord** - Main community  
- **0xPARC Discord** - ZK research/education
- **PSE Discord** - Privacy & Scaling Explorations

**Forums:**
- **Ethereum Research (ZK category)**  
  https://ethresear.ch/c/zero-knowledge/

- **Zcash Forum (ZK discussions)**  
  https://forum.zcashcommunity.com/

**Twitter:**
- @zkemail_
- @gubsheep (lead dev)
- @yush_g (contributor)
- @privacy_scaling (PSE team)

### Production Examples

**Live Projects:**
1. **Proof of Email** - zkEmail demo  
   https://prove.email

2. **Sybil Resistance** - Email-based identity  
   https://github.com/zk-email/email-sybil

3. **zkEmail Recovery** - Wallet recovery via email  
   https://github.com/zk-email/email-recovery

4. **Twitter Verification** - Prove Twitter account via email  
   https://github.com/zk-email/twitter-email-verifier

---

## Conclusion

zkEmail represents a powerful primitive for bringing real-world identity and data on-chain in a privacy-preserving way. By leveraging existing DKIM infrastructure, it enables cryptographic proofs of email contents without requiring new protocols or trust assumptions.

**Key Takeaways:**

1. **DKIM is a gift:** Email authentication that's already deployed globally
2. **RSA is expensive:** 500K+ constraints, main bottleneck
3. **Privacy vs Performance:** Trade-off between what you hide and circuit size
4. **Trust DKIM keys:** Need reliable key registry, track rotations
5. **Many applications:** Recovery, identity, credentials, financial proofs

**When to use zkEmail:**
- ✅ Need proof of email-based identity
- ✅ Want privacy (hide contents)
- ✅ Async verification (email already received)
- ✅ Trust email provider (Gmail, etc.)

**When NOT to use:**
- ❌ Need real-time verification (use zk-TLS instead)
- ❌ Email provider untrustworthy
- ❌ Quantum threat immediate (RSA vulnerable)
- ❌ Can't afford 30s+ proving time

**The future:** Post-quantum DKIM, better browser UX, cross-chain proofs, and integration with account abstraction will make zkEmail even more powerful and accessible.

---

*Last updated: December 2025*  
*For questions or contributions: https://github.com/zk-email/zk-email-verify*
