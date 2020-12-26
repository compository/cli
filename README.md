# Compository Publish CLI

## Publishing a `.dna.workdir` directory

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

This will publish the `profiles` zome with its wasm code, associating it with the UI elements defined in the `bundle.js` file. Note that the UI bundle needs to be a standalone ES module interpretable directly by the browser.

For now this CLI will publish all the zomes included in the json file, althought this might change in the future.

## Building

```bash
cargo build --release
```
