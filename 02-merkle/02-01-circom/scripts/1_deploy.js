import hre from "hardhat";
import fs from "fs";

async function main() {
    console.log("Deploying contracts...\n");
    
    // 1. Deploy PoseidonT3 library
    console.log("Deploying PoseidonT3 library...");
    const PoseidonT3 = await hre.ethers.getContractFactory("PoseidonT3");
    const poseidonT3 = await PoseidonT3.deploy();
    await poseidonT3.waitForDeployment();
    const poseidonT3Address = await poseidonT3.getAddress();
    console.log("✅ PoseidonT3 deployed:", poseidonT3Address);
    
    // 2. Deploy IncrementalMerkleTree
    console.log("\nDeploying IncrementalMerkleTree...");
    const Tree = await hre.ethers.getContractFactory("IncrementalMerkleTree", {
        libraries: {
            "contracts/PoseidonT3.sol:PoseidonT3": poseidonT3Address
        }
    });
    const tree = await Tree.deploy();
    await tree.waitForDeployment();
    const treeAddress = await tree.getAddress();
    console.log("✅ MerkleTree deployed:", treeAddress);
    
    // 3. Deploy Groth16Verifier
    console.log("\nDeploying Groth16Verifier...");
    const Verifier = await hre.ethers.getContractFactory("Groth16Verifier");
    const verifier = await Verifier.deploy();
    await verifier.waitForDeployment();
    const verifierAddress = await verifier.getAddress();
    console.log("✅ Verifier deployed:", verifierAddress);
    
    // 4. Deploy MerkleProofVerifier
    console.log("\nDeploying MerkleProofVerifier...");
    const ProofVerifier = await hre.ethers.getContractFactory("MerkleProofVerifier");
    const proofVerifier = await ProofVerifier.deploy(verifierAddress, treeAddress);
    await proofVerifier.waitForDeployment();
    const proofVerifierAddress = await proofVerifier.getAddress();
    console.log("✅ MerkleProofVerifier deployed:", proofVerifierAddress);
    
    // Save addresses
    fs.writeFileSync("deployed.json", JSON.stringify({
        poseidonT3: poseidonT3Address,
        tree: treeAddress,
        verifier: verifierAddress,
        proofVerifier: proofVerifierAddress
    }, null, 2));
    
    console.log("\n✅ Saved addresses to deployed.json");
}

main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});
