import hre from "hardhat";
import { buildPoseidon } from "circomlibjs";
import fs from "fs";

async function main() {
    const deployed = JSON.parse(fs.readFileSync("deployed.json"));
    const tree = await hre.ethers.getContractAt("IncrementalMerkleTree", deployed.tree);
    
    const secret = Math.floor(Math.random() * 1000000);
    console.log(`Your secret: ${secret}`);
    console.log("⚠️  SAVE THIS!\n");
    
    const poseidon = await buildPoseidon();
    const F = poseidon.F;
    const commitment = poseidon([secret]);
    const commitmentStr = F.toString(commitment);
    
    console.log(`Commitment: ${commitmentStr}\n`);
    
    console.log("Registering on-chain...");
    const tx = await tree.insert(commitmentStr);
    const receipt = await tx.wait();
    
    const event = receipt.logs.find(log => {
        try {
            return tree.interface.parseLog(log).name === "LeafInserted";
        } catch (e) {
            return false;
        }
    });
    
    const parsedEvent = tree.interface.parseLog(event);
    const leafIndex = parsedEvent.args.index;
    
    console.log(`✅ Registered! Leaf index: ${leafIndex}`);
    
    fs.writeFileSync("user.json", JSON.stringify({
        secret: secret,
        commitment: commitmentStr,
        leafIndex: Number(leafIndex)
    }, null, 2));
    
    console.log("✅ Saved to user.json");
}

main().catch(console.error);
