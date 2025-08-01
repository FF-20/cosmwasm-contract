# To Deploy:
- cargo build --release --target wasm32-unknown-unknown
- docker run --rm -v "$(pwd)":/code   --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target   --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry   cosmwasm/optimizer:0.17.0
- go to https://neutron.celat.one/pion-1 
- open the /artifacts folder, and upload the cosmos_cw.wasm file to the website above.