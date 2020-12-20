use holo_hash::AgentPubKey;
use holochain::conductor::api::error::ExternalApiWireError;
use holochain_types::{app::{InstalledApp, InstalledAppId}, cell::CellId};
use holochain_zome_types::{
    capability::CapSecret,
    zome::{FunctionName, ZomeName},
};
use holochain_serialized_bytes::prelude::*;

/// Represents the available Conductor functions to call over an App interface
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SerializedBytes)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum ClientAppRequest {
    AppInfo {
        /// The InstalledAppId for which to get information
        installed_app_id: InstalledAppId,
    },
    ZomeCall(ClientZomeCall),
}

/// The data provided across an App interface in order to make a zome call
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SerializedBytes)]
pub struct ClientZomeCall {
    /// The Id of the `Cell` containing the Zome to be called
    pub cell_id: CellId,
    /// The Zome containing the function to be called
    pub zome_name: ZomeName,
    /// The name of the Zome function to call
    pub fn_name: FunctionName,
    /// The serialized data to pass as an argument to the Zome call
    pub payload: SerializedBytes,
    /// The capability request authorization.
    /// This can be `None` and still succeed in the case where the function
    /// in the zome being called has been given an Unrestricted status
    /// via a `CapGrant`. Otherwise, it will be necessary to provide a `CapSecret` for every call.
    pub cap: Option<CapSecret>,
    /// The provenance (source) of the call.
    ///
    /// NB: **This will go away** as soon as Holochain has a way of determining who
    /// is making this ZomeCall over this interface. Until we do, the caller simply
    /// provides this data and Holochain trusts them.
    pub provenance: AgentPubKey,
}


/// Responses to requests received on an App interface
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SerializedBytes)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum ClientAppResponse {
    Error(ExternalApiWireError),

    AppInfo(Option<InstalledApp>),

    ZomeCall(SerializedBytes),
}