import { readFileSync, writeFileSync } from 'fs';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { GasPrice } from '@cosmjs/stargate';
import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';

// Use EXACT versions to avoid type conflicts (package.json):
// "@cosmjs/cosmwasm-stargate": "0.32.2"
// "@cosmjs/stargate": "0.32.2"
// "@cosmjs/proto-signing": "0.32.2"

interface DeploymentInfo {
  network: string;
  codeId: number;
  contractAddress: string;
  deployer: string;
  transactionHash: string;
  deployedAt: string;
}

interface WalletConfig {
  name: string;
  type: string;
  address: string;
  pubkey: string;
  mnemonic: string;
}

export default async function deployWithLocalWallet(): Promise<void> {
  // Network configuration
  const NETWORK = 'neutron-testnet';
  const RPC_ENDPOINT = 'https://rpc-palvus.pion-1.ntrn.tech:443';
  const CHAIN_ID = 'pion-1';

  // Your wallet configuration from wasmkit.config.js
  const walletConfig: WalletConfig = {
    name: "neutron-deploy",
    type: "local",
    address: "neutron15jp6un07vsjcukafcrql48hnec6e2hjpgjkakx",
    pubkey: "{\"@type\":\"/cosmos.crypto.secp256k1.PubKey\",\"key\":\"A7DGbatH1k59aGw3rol5VDVjvYf469uKYaXrCKolBQhF\"}",
    mnemonic: "height idle output caution catch word tower mention door upgrade denial thunder matter appear order learn project olympic miracle nerve exhibit ozone process aspect"
  };

  console.log(`Using local wallet: ${walletConfig.address}`);

  // 1. Create wallet from mnemonic
  let wallet: DirectSecp256k1HdWallet;
  try {
    wallet = await DirectSecp256k1HdWallet.fromMnemonic(
      walletConfig.mnemonic,
      {
        prefix: "neutron", // Neutron address prefix
      }
    );
  } catch (error) {
    throw new Error(`Failed to create wallet from mnemonic: ${error instanceof Error ? error.message : String(error)}`);
  }

  // 2. Verify wallet address matches config
  const accounts = await wallet.getAccounts();
  const deployerAddress = accounts[0]?.address;

  if (!deployerAddress) {
    throw new Error('Failed to get account from wallet');
  }

  if (deployerAddress !== walletConfig.address) {
    console.warn(`⚠️  Address mismatch!`);
    console.warn(`Config address: ${walletConfig.address}`);
    console.warn(`Derived address: ${deployerAddress}`);
    console.warn(`Using derived address: ${deployerAddress}`);
  }

  console.log(`Deployer address: ${deployerAddress}`);

  // 3. Create signing client
  const gasPrice = GasPrice.fromString('0.025untrn');
  let client: SigningCosmWasmClient;
  
  try {
    client = await SigningCosmWasmClient.connectWithSigner(
      RPC_ENDPOINT,
      wallet,
      {
        gasPrice: gasPrice,
        broadcastTimeoutMs: 30000, // 30 seconds timeout
        broadcastPollIntervalMs: 1000, // Check every second
      }
    );
  } catch (error) {
    throw new Error(`Failed to connect to RPC: ${error instanceof Error ? error.message : String(error)}`);
  }

  // 4. Check account balance
  try {
    const balance = await client.getBalance(deployerAddress, "untrn");
    console.log(`Account balance: ${balance.amount} ${balance.denom}`);
    
    // Convert balance to NTRN (6 decimals)
    const balanceInNtrn = parseFloat(balance.amount) / 1_000_000;
    console.log(`Balance: ${balanceInNtrn} NTRN`);

    if (balanceInNtrn < 0.1) {
      console.warn('⚠️  Low balance! You may need more NTRN for deployment.');
    }
  } catch (error) {
    console.warn(`Could not fetch balance: ${error instanceof Error ? error.message : String(error)}`);
  }

  // 5. Load WASM file
  const wasmPath = 'artifacts/contracts/cosmos_cw.wasm';
  let wasmBytecode: Uint8Array;
  
  try {
    const buffer = readFileSync(wasmPath);
    wasmBytecode = new Uint8Array(buffer);
    console.log(`Loaded WASM file: ${wasmBytecode.length} bytes`);
  } catch (error) {
    throw new Error(`Failed to read WASM file at '${wasmPath}': ${error instanceof Error ? error.message : String(error)}`);
  }

  // 6. Upload contract
  console.log('📦 Uploading contract to blockchain...');
  let uploadResult;
  
  try {
    uploadResult = await client.upload(
      deployerAddress,
      wasmBytecode,
      'auto', // Let the client estimate gas
      'Contract deployment via local wallet'
    );
  } catch (error) {
    throw new Error(`Upload failed: ${error instanceof Error ? error.message : String(error)}`);
  }

  const codeId = uploadResult.codeId;
  console.log(`✅ Code uploaded successfully!`);
  console.log(`📋 Code ID: ${codeId}`);
  console.log(`🔗 Upload transaction: ${uploadResult.transactionHash}`);

  // 7. Instantiate contract
  console.log('🏗️  Instantiating contract...');
  
  // Customize your initialization message here
  const initMsg = {
    // Add your contract's initialization parameters here
    // Example:
    // owner: deployerAddress,
    // name: "My Contract",
    // symbol: "MCT"
  };

  let instantiateResult;
  
  try {
    instantiateResult = await client.instantiate(
      deployerAddress,
      codeId,
      initMsg,
      'My Contract Instance', // Contract label - customize as needed
      'auto', // Let client estimate gas
      {
        admin: deployerAddress, // Set yourself as admin (optional)
        memo: 'Contract instantiation via local wallet'
      }
    );
  } catch (error) {
    throw new Error(`Instantiation failed: ${error instanceof Error ? error.message : String(error)}`);
  }

  if (!instantiateResult.contractAddress) {
    throw new Error('Contract address not returned from instantiation!');
  }

  console.log(`🎉 Contract deployed successfully!`);
  console.log(`📍 Contract address: ${instantiateResult.contractAddress}`);
  console.log(`🔗 Instantiate transaction: ${instantiateResult.transactionHash}`);

  // 8. Save deployment information
  const deploymentInfo: DeploymentInfo = {
    network: NETWORK,
    codeId,
    contractAddress: instantiateResult.contractAddress,
    deployer: deployerAddress,
    transactionHash: instantiateResult.transactionHash,
    deployedAt: new Date().toISOString()
  };

  const deploymentFile = `deployment-${NETWORK}.json`;
  
  try {
    writeFileSync(
      deploymentFile,
      JSON.stringify(deploymentInfo, null, 2)
    );
    console.log(`💾 Deployment info saved to: ${deploymentFile}`);
  } catch (error) {
    console.warn(`Could not save deployment file: ${error instanceof Error ? error.message : String(error)}`);
  }
  
  console.log('\n🚀 Deployment Summary:');
  console.log('='.repeat(50));
  console.log(`Network: ${NETWORK}`);
  console.log(`Code ID: ${codeId}`);
  console.log(`Contract Address: ${instantiateResult.contractAddress}`);
  console.log(`Deployer: ${deployerAddress}`);
  console.log(`Deployed At: ${deploymentInfo.deployedAt}`);
  console.log('='.repeat(50));
}

// Error handling wrapper
async function main() {
  try {
    await deployWithLocalWallet();
    console.log('\n✅ Deployment completed successfully!');
  } catch (error) {
    console.error('\n❌ Deployment failed!');
    console.error('Error:', error instanceof Error ? error.message : String(error));
    
    if (error instanceof Error && error.stack) {
      console.error('\nStack trace:');
      console.error(error.stack);
    }
    
    console.error('\n💡 Troubleshooting tips:');
    console.error('- Check your internet connection');
    console.error('- Verify the WASM file exists at artifacts/contract.wasm');
    console.error('- Ensure your wallet has sufficient NTRN balance');
    console.error('- Try again in a few minutes if RPC is temporarily unavailable');
    
    process.exit(1);
  }
}

// Run the deployment
main();