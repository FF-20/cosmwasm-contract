"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.default = deployWithKeplr;
const fs_1 = require("fs");
const cosmwasm_stargate_1 = require("@cosmjs/cosmwasm-stargate");
const stargate_1 = require("@cosmjs/stargate");
async function deployWithKeplr() {
    // Network configuration
    const NETWORK = 'neutron-testnet';
    const RPC_ENDPOINT = 'https://rpc-palvus.pion-1.ntrn.tech:443';
    const CHAIN_ID = 'pion-1';
    // 1. Keplr check with proper type guard
    if (!window.keplr) {
        throw new Error('Keplr extension not found!');
    }
    // 2. Suggest chain with error handling
    try {
        await window.keplr.experimentalSuggestChain({
            chainId: CHAIN_ID,
            chainName: 'Neutron Testnet',
            rpc: RPC_ENDPOINT,
            rest: 'https://rest-palvus.pion-1.ntrn.tech',
            bip44: { coinType: 118 },
            bech32Config: {
                bech32PrefixAccAddr: 'neutron',
                bech32PrefixAccPub: 'neutronpub',
                bech32PrefixValAddr: 'neutronvaloper',
                bech32PrefixValPub: 'neutronvaloperpub',
                bech32PrefixConsAddr: 'neutronvalcons',
                bech32PrefixConsPub: 'neutronvalconspub',
            },
            currencies: [{
                    coinDenom: 'NTRN',
                    coinMinimalDenom: 'untrn',
                    coinDecimals: 6,
                }],
            feeCurrencies: [{
                    coinDenom: 'NTRN',
                    coinMinimalDenom: 'untrn',
                    coinDecimals: 6,
                    gasPriceStep: {
                        low: 0.01,
                        average: 0.025,
                        high: 0.05,
                    },
                }],
            stakeCurrency: {
                coinDenom: 'NTRN',
                coinMinimalDenom: 'untrn',
                coinDecimals: 6,
            },
            features: ['stargate', 'ibc-transfer', 'cosmwasm'],
        });
    }
    catch (error) {
        console.warn('Chain suggestion error:', error instanceof Error ? error.message : String(error));
    }
    // 3. Enable Keplr with proper error handling
    try {
        await window.keplr.enable(CHAIN_ID);
    }
    catch (error) {
        throw new Error(`Keplr enable failed: ${error instanceof Error ? error.message : String(error)}`);
    }
    // 4. Get accounts with type safety
    const offlineSigner = window.keplr.getOfflineSigner(CHAIN_ID);
    const accounts = await offlineSigner.getAccounts();
    if (!accounts || accounts.length === 0) {
        throw new Error('No accounts found in Keplr wallet!');
    }
    const deployerAddress = accounts[0]?.address;
    if (!deployerAddress) {
        throw new Error('Account address is undefined!');
    }
    console.log(`Using Keplr wallet: ${deployerAddress}`);
    // 5. Create client with GasPrice object (with type assertion if needed)
    const gasPrice = stargate_1.GasPrice.fromString('0.025untrn');
    const client = await cosmwasm_stargate_1.SigningCosmWasmClient.connectWithSigner(RPC_ENDPOINT, offlineSigner, {
        gasPrice: gasPrice // Type assertion to bypass version conflicts
    });
    // 6. Load WASM with error handling
    const wasmPath = 'artifacts/contract.wasm';
    let wasmBytecode;
    try {
        const buffer = (0, fs_1.readFileSync)(wasmPath);
        wasmBytecode = new Uint8Array(buffer);
    }
    catch (error) {
        throw new Error(`Failed to read WASM file: ${error instanceof Error ? error.message : String(error)}`);
    }
    console.log(`Loaded WASM file: ${wasmBytecode.length} bytes`);
    // 7. Upload contract
    console.log('Uploading contract...');
    const uploadResult = await client.upload(deployerAddress, wasmBytecode, 'auto');
    const codeId = uploadResult.codeId;
    console.log(`Code uploaded with ID: ${codeId}`);
    console.log(`Upload transaction: ${uploadResult.transactionHash}`);
    // 8. Instantiate contract
    console.log('Instantiating contract...');
    const initMsg = {}; // Your initialization message - update this as needed
    const instantiateResult = await client.instantiate(deployerAddress, codeId, initMsg, 'My Contract', // contract label
    'auto', // fee
    {
        admin: deployerAddress // optional admin
    });
    if (!instantiateResult.contractAddress) {
        throw new Error('Contract address not returned from instantiation!');
    }
    console.log(`Contract instantiated at: ${instantiateResult.contractAddress}`);
    console.log(`Instantiate transaction: ${instantiateResult.transactionHash}`);
    // 9. Save deployment info
    const deploymentInfo = {
        network: NETWORK,
        codeId,
        contractAddress: instantiateResult.contractAddress,
        deployer: deployerAddress,
        transactionHash: instantiateResult.transactionHash
    };
    const deploymentFile = `deployment-${NETWORK}.json`;
    (0, fs_1.writeFileSync)(deploymentFile, JSON.stringify(deploymentInfo, null, 2));
    console.log('Deployment successful!');
    console.log(`Deployment info saved to: ${deploymentFile}`);
    console.log('Deployment details:', deploymentInfo);
}
// Error handling wrapper
async function main() {
    try {
        await deployWithKeplr();
    }
    catch (error) {
        console.error('Deployment failed:', error instanceof Error ? error.message : String(error));
        if (error instanceof Error && error.stack) {
            console.error('Stack trace:', error.stack);
        }
        process.exit(1);
    }
}
// Browser context check
if (typeof window !== 'undefined' && window.keplr) {
    main();
}
else {
    console.error('This script requires browser environment with Keplr extension');
    console.error('Make sure to run this in a browser with Keplr installed');
}
//# sourceMappingURL=deploy.js.map