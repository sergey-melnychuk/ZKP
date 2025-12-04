import hre from "hardhat";
import { buildPoseidon } from "circomlibjs";
import fs from "fs";
import { exec } from "child_process";
import util from "util";

const execPromise = util.promisify(exec);

async function main() {
    if (!fs.existsSync("deployed.json")) {
        console.error("❌ deployed.json not found. Run 1_deploy.js first!");
        process.exit(1);
    }
    if (!fs.existsSync("user.json")) {
        console.error("❌ user.json not found. Run 2_register.js first!");
        process.exit(1);
    }
    
    const deployed = JSON.parse(fs.readFileSync("deployed.json"));
    const user = JSON.parse(fs.readFileSync("user.json"));
    
    const tree = await hre.ethers.getContractAt("IncrementalMerkleTree", deployed.tree);
    
    const nextIndex = await tree.nextIndex();
    console.log(`Total leaves in tree: ${nextIndex}`);
    console.log(`Your leaf index: ${user.leafIndex}`);
    
    if (user.leafIndex >= nextIndex) {
        console.error(`❌ Leaf index ${user.leafIndex} not found in tree (only ${nextIndex} leaves)`);
        process.exit(1);
    }
    
    console.log("\nGetting Merkle path from contract...");
    const result = await tree.getPath(user.leafIndex);
    const siblings = result[0].map(s => s.toString());
    const pathIndices = result[1].map(i => Number(i));
    
    console.log(`Path indices: ${pathIndices.join(', ')}`);
    
    const currentRoot = await tree.root();
    console.log(`Root: ${currentRoot.toString()}\n`);
    
    const poseidon = await buildPoseidon();
    const F = poseidon.F;
    const nullifier = poseidon([user.secret]);
    
    const input = {
        secret: user.secret.toString(),
        siblings: siblings,
        pathIndices: pathIndices,
        root: currentRoot.toString(),
        nullifier: F.toString(nullifier)
    };
    
    fs.writeFileSync("input.json", JSON.stringify(input, null, 2));
    console.log("✅ Created input.json");
    
    console.log("\nGenerating witness...");
    // Use snarkjs instead of node generate_witness.js
    await execPromise("snarkjs wtns calculate merkle_js/merkle.wasm input.json witness.wtns");
    console.log("✅ Witness generated");
    
    console.log("\nGenerating proof...");
    await execPromise("snarkjs groth16 prove merkle_final.zkey witness.wtns proof.json public.json");
    console.log("✅ Proof generated");
    
    console.log("\nVerifying proof locally...");
    const { stdout } = await execPromise("snarkjs groth16 verify verification_key.json public.json proof.json");
    console.log(stdout);
}

main().catch(console.error);
