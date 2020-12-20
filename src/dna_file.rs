use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use holochain::core::ribosome::{
    guest_callback::entry_defs::{EntryDefsHostAccess, EntryDefsInvocation, EntryDefsResult},
    real_ribosome::RealRibosome,
    RibosomeT,
};
use holochain_serialized_bytes::prelude::*;
use holochain_types::{
    dna::zome::WasmZome,
    dna::{wasm::DnaWasm, DnaDef, DnaFile},
    prelude::SerializedBytes,
};
use holochain_zome_types::{
    entry_def::{EntryDefId, EntryDefs},
    zome::ZomeName,
};
use std::convert::TryInto;

use crate::types::ZomeWithCode;

pub async fn read_dna(dna_work_dir: &impl AsRef<std::path::Path>) -> Result<DnaDefJson> {
    let dna_work_dir = dna_work_dir.as_ref().canonicalize()?;
    let mut json_filename = dna_work_dir.clone();
    json_filename.push("dna.json");

    let json_data = tokio::fs::read(json_filename.clone()).await?;

    let json_file: DnaDefJson = serde_json::from_slice(&json_data)?;

    Ok(json_file)
}

pub fn get_entry_defs(dna_file: DnaFile) -> Result<BTreeMap<ZomeName, EntryDefs>> {
    let ribosome = RealRibosome::new(dna_file);

    let entry_defs = ribosome.run_entry_defs(EntryDefsHostAccess, EntryDefsInvocation)?;

    match entry_defs {
        EntryDefsResult::Defs(defs) => Ok(defs),
        EntryDefsResult::Err(_, __) => Err(anyhow!("Could not get entry defs")),
    }
}

pub async fn get_zomes(
    dna_file_content: &DnaDefJson,
    dna_work_dir: &impl AsRef<std::path::Path>,
) -> Result<Vec<ZomeWithCode>> {
    let dna_work_dir = dna_work_dir.as_ref().canonicalize()?;

    let dna_file = dna_file_content.compile_dna_file(&dna_work_dir).await?;

    let (dna_def, _): (DnaDef, Vec<DnaWasm>) = dna_file.clone().into();
    let dna_code = dna_file.code().clone();

    let entry_defs = get_entry_defs(dna_file)?;

    let mut zomes: Vec<ZomeWithCode> = vec![];

    for (zome_name, zome_entry_defs) in entry_defs.into_iter() {
        let wasm_zome = dna_def.get_wasm_zome(&zome_name)?;

        let wasm_code = dna_code
            .get(&wasm_zome.wasm_hash)
            .ok_or(anyhow!("Bad dna file"))?;

        let str_entry_defs = zome_entry_defs
            .into_iter()
            .map(|entry_def| match entry_def.id {
                EntryDefId::App(entry_def_id) => Ok(entry_def_id),
                _ => Err(anyhow!("Bad entry def")),
            })
            .collect::<Result<Vec<String>>>()?;

        let components_bundle = match dna_file_content.zomes.get(&zome_name) {
            Some(zome_json) => match zome_json.ui_path.clone() {
                Some(ui_path) => {
                    let mut zome_file_path = dna_work_dir.clone();
                    zome_file_path.push(&ui_path);

                    let file_contents = tokio::fs::read(zome_file_path.clone()).await?;
                    Some(file_contents)
                }
                None => None,
            },
            None => None,
        };

        zomes.push(ZomeWithCode {
            name: zome_name.0,
            components_bundle,
            wasm_code: wasm_code.clone(),
            wasm_hash: wasm_zome.wasm_hash.clone(),
            entry_defs: str_entry_defs,
            required_properties: vec![],
            required_membrane_proof: false,
        });
    }

    Ok(zomes)
}

/// See `holochain_types::dna::zome::Zome`.
/// This is a helper to convert to json.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ZomeJson {
    pub wasm_path: String,
    pub ui_path: Option<String>,
}

/// Special Json Value Decode Helper
#[derive(Debug, serde::Serialize, serde::Deserialize, SerializedBytes)]
struct JsonValueDecodeHelper(pub serde_json::Value);

/// See `holochain_types::dna::DnaDef`.
/// This is a helper to convert to json.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DnaDefJson {
    pub name: String,
    pub uuid: String,
    pub properties: serde_json::Value,
    pub zomes: BTreeMap<ZomeName, ZomeJson>,
    pub ui_path: Option<String>,
}

impl DnaDefJson {
    pub async fn compile_dna_file(
        &self,
        work_dir: impl Into<std::path::PathBuf>,
    ) -> Result<DnaFile> {
        let work_dir = work_dir.into();

        let properties: SerializedBytes =
            JsonValueDecodeHelper(self.properties.clone()).try_into()?;

        let mut zomes = Vec::new();
        let mut wasm_list = Vec::new();

        for (zome_name, zome) in self.zomes.iter() {
            let mut zome_file_path = work_dir.clone();
            zome_file_path.push(&zome.wasm_path);

            let zome_content = tokio::fs::read(zome_file_path).await?;

            let wasm: DnaWasm = zome_content.into();
            let wasm_hash = holo_hash::WasmHash::with_data(&wasm).await;
            zomes.push((zome_name.clone(), WasmZome { wasm_hash }.into()));
            wasm_list.push(wasm);
        }

        let dna = DnaDef {
            name: self.name.clone(),
            uuid: self.uuid.clone(),
            properties,
            zomes,
        };

        Ok(DnaFile::new(dna, wasm_list).await?)
    }
}
