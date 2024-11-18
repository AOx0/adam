use crate::{AppState, Ip};
use axum::{async_trait, extract::FromRequestParts, http::request::Parts, response::Redirect};

pub struct Selected(pub Ip);

#[async_trait]
impl FromRequestParts<AppState> for Selected {
    type Rejection = Redirect;

    async fn from_request_parts(_: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        Ok(Selected(
            (state.inner.selected_ip.read().await.clone()).ok_or(Redirect::to("/ips/add"))?,
        ))
    }
}
