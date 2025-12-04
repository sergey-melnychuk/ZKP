import hre from "hardhat";
import fs from "fs";

async function main() {
    const deployed = JSON.parse(fs.readFileSync("deployed.json"));
    const proof = JSON.parse(fs.readFileSync("proof.json"));
    const publicSignals = JSON.parse(fs.readFileSync("public.json"));
    
    const verifier = await hre.ethers.getContractAt("MerkleProofVerifier", deployed.proofVerifier);
    
    // Format proof
    const pA = [proof.pi_a[0], proof.pi_a[1]];
    const pB = [[proof.pi_b[0][1], proof.pi_b[0][0]], [proof.pi_b[1][1], proof.pi_b[1][0]]];
    const pC = [proof.pi_c[0], proof.pi_c[1]];
    const nullifier = publicSignals[1];
    
    console.log("Submitting proof on-chain...");
    // Use submitProof instead of verify
    const tx = await verifier.submitProof(pA, pB, pC, nullifier);
    const receipt = await tx.wait();
    
    console.log("✅ Proof verified on-chain!");
    console.log(`Transaction: ${receipt.hash}`);
}

main().catch(console.error);
