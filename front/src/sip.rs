use crate::AppState;
use axum::{async_trait, extract::FromRequestParts, http::request::Parts, response::Redirect};
use std::net::SocketAddr;

pub struct Selected(pub SocketAddr);

#[async_trait]
impl FromRequestParts<AppState> for Selected {
    type Rejection = Redirect;

    async fn from_request_parts(_: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        Ok(Selected(
            (*state.inner.selected_ip.read().await).ok_or(Redirect::to("/ips/add"))?,
        ))
    }
}
