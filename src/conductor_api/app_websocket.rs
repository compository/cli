use anyhow::{anyhow, Context, Result};
use holochain_types::cell::CellId;
use holochain_websocket::{websocket_connect, WebsocketConfig, WebsocketSender};
use std::sync::Arc;
use tracing::{instrument, trace};
use url::Url;

use super::{types::{ClientAppRequest, ClientAppResponse, ClientZomeCall}};

#[derive(Clone)]
pub struct AppWebsocket {
    tx: WebsocketSender,
}

impl AppWebsocket {
    #[instrument(err)]
    pub async fn connect(url: String) -> Result<Self> {
        let url = Url::parse(&url).context("invalid ws:// URL")?;
        let websocket_config = Arc::new(WebsocketConfig::default());
        let (tx, _rx) = websocket_connect(url.clone().into(), websocket_config).await?;
        Ok(Self { tx })
    }

    #[instrument(skip(self), err)]
    pub async fn app_info(&mut self, installed_app_id: String) -> Result<ClientAppResponse> {
        self.send(ClientAppRequest::AppInfo { installed_app_id })
            .await
    }

    #[instrument(skip(self), err)]
    pub async fn call_zome(
        &mut self,
        cell_id: &CellId,
        call: ClientZomeCall,
    ) -> Result<ClientAppResponse> {
        self.send(ClientAppRequest::ZomeCall(call)).await
    }

    async fn send(&mut self, msg: ClientAppRequest) -> Result<ClientAppResponse> {
        let response = self
            .tx
            .request(msg)
            .await
            .context("failed to send message")?;
        match response {
            ClientAppResponse::Error(error) => Err(anyhow!("error: {:?}", error)),
            _ => {
                trace!("send successful");
                Ok(response)
            }
        }
    }
}
