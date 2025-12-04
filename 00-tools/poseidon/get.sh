#!/bin/sh

npm i poseidon-solidity

rm *.sol

cp node_modules/poseidon-solidity/PoseidonT3.sol .

rm -rf node_modules/
rm package.json
rm package-lock.json
