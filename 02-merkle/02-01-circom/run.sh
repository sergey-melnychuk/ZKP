#!/bin/sh

npm install
npx hardhat compile

circom circuits/merkle.circom --r1cs --wasm --sym -l node_modules

killall node
npx hardhat node &
sleep 2

npx hardhat run scripts/1_deploy.js --network localhost
npx hardhat run scripts/2_register.js --network localhost
npx hardhat run scripts/3_prove.js --network localhost
npx hardhat run scripts/4_submit.js --network localhost

killall node
