const networks = {
  testnet: {
    endpoint: "https://rpc-palvus.pion-1.ntrn.tech/",
    chainId: "pion-1",
    accounts: [{ 
        name: "neutron-deploy", 
        address: "neutron15jp6un07vsjcukafcrql48hnec6e2hjpgjkakx", 
        mnemonic: "height idle output caution catch word tower mention door upgrade denial thunder matter appear order learn project olympic miracle nerve exhibit ozone process aspect" 
    }],
  },
};
module.exports = { networks: { default: networks.testnet, testnet: networks.testnet } };