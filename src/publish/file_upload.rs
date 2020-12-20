use holochain_serialized_bytes::prelude::*;
use std::{convert::TryInto, time::SystemTime};

use anyhow::{anyhow, Result};
use hc_utils::WrappedEntryHash;
use holochain_types::cell::CellId;
use holochain_zome_types::timestamp;

use crate::conductor_api::{
    app_websocket::AppWebsocket,
    types::{ClientAppResponse, ClientZomeCall},
};

const CHUNKS_SIZE: usize = 1024 * 1024 * 10;

pub async fn upload_file(
    ws: &mut AppWebsocket,
    compository_cell_id: &CellId,
    name: String,
    file_type: String,
    content: &[u8],
) -> Result<String> {
    let size = content.len();
    let chunk_iter = content.chunks(CHUNKS_SIZE);

    let mut chunk_hashes: Vec<String> = vec![];

    for chunk in chunk_iter {
        let hash = upload_chunk(ws, &compository_cell_id, chunk).await?;

        chunk_hashes.push(hash);
    }

    let file_hash = create_file(
        ws,
        &compository_cell_id,
        name,
        file_type,
        size,
        chunk_hashes,
    )
    .await?;

    Ok(file_hash)
}

#[derive(Debug, Clone, Serialize, Deserialize, SerializedBytes)]
struct Chunk(Vec<u8>);

async fn upload_chunk(
    ws: &mut AppWebsocket,
    compository_cell_id: &CellId,
    code: &[u8],
) -> Result<String> {
    let zome_call = ClientZomeCall {
        cap: None,
        cell_id: compository_cell_id.clone(),
        fn_name: "create_file_chunk".into(),
        payload: Chunk(code.to_vec()).try_into()?,
        provenance: compository_cell_id.agent_pubkey().clone(),
        zome_name: "file_storage".into(),
    };
    let response = ws.call_zome(compository_cell_id, zome_call).await?;

    match response {
        ClientAppResponse::ZomeCall(bytes) => {
            let hash: WrappedEntryHash = bytes.try_into()?;

            Ok(format!("{}", hash.0))
        }
        _ => Err(anyhow!("Bad response")),
    }
}

#[derive(Clone, Serialize, Deserialize, SerializedBytes)]
#[serde(rename_all = "camelCase")]
pub struct CreateFileMetadataInput {
    pub name: String,
    pub last_modified: timestamp::Timestamp,
    pub size: usize,
    pub file_type: String,
    pub chunks_hashes: Vec<String>,
}

async fn create_file(
    ws: &mut AppWebsocket,
    compository_cell_id: &CellId,
    name: String,
    file_type: String,
    size: usize,
    chunks_hashes: Vec<String>,
) -> Result<String> {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(SystemTime::UNIX_EPOCH)?;

    let timestamp = timestamp::Timestamp(
        since_the_epoch.as_secs() as i64,
        since_the_epoch.subsec_nanos(),
    );

    let payload = CreateFileMetadataInput {
        name,
        last_modified: timestamp,
        size,
        chunks_hashes,
        file_type,
    };

    let zome_call = ClientZomeCall {
        cap: None,
        cell_id: compository_cell_id.clone(),
        fn_name: "create_file_metadata".into(),
        payload: payload.try_into()?,
        provenance: compository_cell_id.agent_pubkey().clone(),
        zome_name: "file_storage".into(),
    };

    let response = ws.call_zome(compository_cell_id, zome_call).await?;

    match response {
        ClientAppResponse::ZomeCall(bytes) => {
            let hash: WrappedEntryHash = bytes.try_into()?;

            Ok(format!("{}", hash.0))
        }
        _ => Err(anyhow!("Bad response")),
    }
}
