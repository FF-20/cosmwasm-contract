// Clean browser deployment script without GasPrice import conflicts
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { OfflineSigner } from "@cosmjs/proto-signing";

// Keplr interface
interface Keplr {
  enable(chainId: string): Promise<void>;
  getOfflineSigner(chainId: string): OfflineSigner;
  getKey(chainId: string): Promise<{
    name: string;
    algo: string;
    pubKey: Uint8Array;
    address: Uint8Array;
    bech32Address: string;
  }>;
}

declare global {
  interface Window {
    keplr?: Keplr;
  }
}

export async function deployWithKeplr() {
  try {
    if (!window.keplr) {
      throw new Error("Keplr extension not found. Please install Keplr wallet.");
    }

    const chainId = "pion-1";
    await window.keplr.enable(chainId);

    const offlineSigner = window.keplr.getOfflineSigner(chainId);
    const accounts = await offlineSigner.getAccounts();
    const deployerAddress = accounts[0].address;

    console.log(`🔗 Connected to Keplr`);
    console.log(`Deployer: ${deployerAddress}`);

    // Connect client with string gas price (no import conflicts)
    const client = await SigningCosmWasmClient.connectWithSigner(
      "https://rpc-palvus.pion-1.ntrn.tech/",
      offlineSigner,
      {
        gasPrice: "0.025untrn", // String format avoids version conflicts
        broadcastTimeoutMs: 10000,
        broadcastPollIntervalMs: 500
      }
    );

    // Check balance
    const balance = await client.getBalance(deployerAddress, "untrn");
    console.log(`💰 Balance: ${balance.amount} ${balance.denom}`);

    if (parseFloat(balance.amount) < 1000000) { // Less than 1 NTRN
      console.warn("⚠️  Low balance - you may need more NTRN for deployment");
    }

    // Load WASM file
    console.log("📁 Select your .wasm file...");
    const wasmBytes = await loadWasmFile();
    console.log(`📦 WASM file loaded: ${wasmBytes.length} bytes`);

    // Upload contract
    console.log("🚀 Uploading contract to blockchain...");
    const uploadRes = await client.upload(
      deployerAddress, 
      wasmBytes, 
      "auto",
      "Contract upload via Keplr"
    );
    
    console.log(`✅ Upload successful!`);
    console.log(`📋 Code ID: ${uploadRes.codeId}`);
    console.log(`🔗 Tx: ${uploadRes.transactionHash}`);

    // Instantiate contract
    console.log("🏗️  Creating contract instance...");
    const instantiateRes = await client.instantiate(
      deployerAddress,
      uploadRes.codeId,
      {}, // Empty init message - customize as needed
      "My Contract",
      "auto",
      {
        admin: deployerAddress,
        memo: "Contract instantiation via Keplr"
      }
    );

    console.log(`🎉 Deployment completed!`);
    console.log(`📍 Contract Address: ${instantiateRes.contractAddress}`);
    console.log(`🔗 Tx: ${instantiateRes.transactionHash}`);

    const result = {
      codeId: uploadRes.codeId,
      contractAddress: instantiateRes.contractAddress,
      uploadTxHash: uploadRes.transactionHash,
      instantiateTxHash: instantiateRes.transactionHash,
      deployedAt: new Date().toISOString(),
      deployer: deployerAddress,
      network: "neutron-testnet"
    };

    // Save to localStorage
    localStorage.setItem('neutron-deployment', JSON.stringify(result, null, 2));
    console.log("💾 Deployment info saved to browser storage");

    return result;

  } catch (error) {
    console.error("❌ Deployment failed:", error);
    
    // More specific error messages
    if (error instanceof Error) {
      if (error.message.includes("insufficient funds")) {
        console.error("💸 Not enough NTRN tokens for deployment");
      } else if (error.message.includes("gas")) {
        console.error("⛽ Gas estimation failed - try again");
      } else if (error.message.includes("rejected")) {
        console.error("🚫 Transaction rejected by user");
      }
    }
    
    throw error;
  }
}

// File loader with better UX
async function loadWasmFile(): Promise<Uint8Array> {
  return new Promise((resolve, reject) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.wasm';
    input.style.position = 'fixed';
    input.style.top = '-1000px';
    
    input.onchange = (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) {
        reject(new Error('No file selected'));
        return;
      }

      if (!file.name.endsWith('.wasm')) {
        reject(new Error('Please select a .wasm file'));
        return;
      }

      const reader = new FileReader();
      reader.onload = () => {
        try {
          const bytes = new Uint8Array(reader.result as ArrayBuffer);
          if (bytes.length === 0) {
            reject(new Error('File is empty'));
            return;
          }
          resolve(bytes);
        } catch (err) {
          reject(new Error('Failed to read file as binary'));
        }
      };
      
      reader.onerror = () => reject(new Error('Failed to read file'));
      reader.readAsArrayBuffer(file);
    };
    
    input.oncancel = () => reject(new Error('File selection cancelled'));
    
    document.body.appendChild(input);
    input.click();
    document.body.removeChild(input);
  });
}

// Usage example:
// deployWithKeplr().then(result => {
//   console.log('Deployment successful:', result);
// }).catch(error => {
//   console.error('Deployment failed:', error);
// });