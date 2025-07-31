const networks = {
  testnet: {
    endpoint: "https://rpc-palvus.pion-1.ntrn.tech/",
    chainId: "pion-1",
    accounts: [{ 
        name: "account_0", 
        address: "neutron1syk9g8dc5vyjv82xcxtwt00aek6lv8sw0xywvt", 
        mnemonic: "wire myth slow sauce echo naive broken carry dry mutual giant offer moon clown grape move relief pizza neck salmon dry brother universe kitchen" 
    }],
  },
};
module.exports = { networks: { default: networks.testnet, testnet: networks.testnet } };