# Compository Publish CLI

## How to publish a zome into the compository

1. Run the compository locally DNA with: 
```bash
docker run -it --init -v compository:/database -p 22222:22222 -p 22223:22223 guillemcordoba/compository:0.2
```
2. Install this CLI:
```bash
cargo install --git https://github.com/compository/cli
```

3. Publish your `*.dna.workdir/dna.json` contents:
```bash
./target/release/compository -c uhC0k17jxt5BaQRkGTk2pNbPD7vjL9NPQZwiMTLL5TGWWp2znhbyf -w ~/projects/holochain/blocky/zome/blocky.dna.workdir/ -i test-app -u ws://localhost:8888
```

This is how a `dna.json` looks like when ready to publish:

```json
{
  "name": "profiles",
  "uuid": "",
  "properties": null,
  "zomes": {
    "profiles": {
      "wasm_path": "../target/wasm32-unknown-unknown/release/profiles.wasm",
      "ui_path": "../../ui/bundle/bundle.js"
    }
  }
}
```

This will publish the `profiles` zome with its wasm code, associating it with the UI elements defined in the `bundle.js` file. The `ui_path` property is optional, if not present the zome will be published without any UI associated.

For more information, read [How to create a compository bundle](https://github.com/compository/lib).

For now this CLI will publish all the zomes included in the json file, althought this might change in the future.

## Building

```bash
cargo build --release
```
