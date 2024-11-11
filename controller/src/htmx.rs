use axum::{async_trait, extract::FromRequestParts, http::request::Parts};

#[derive(PartialEq, Eq)]
pub enum Htmx {
    Enabled,
    Disabled,
}

#[allow(dead_code)]
impl Htmx {
    #[inline(always)]
    pub fn enabled(&self) -> bool {
        &Htmx::Enabled == self
    }

    #[inline(always)]
    pub fn disabled(&self) -> bool {
        &Htmx::Disabled == self
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Htmx {
    type Rejection = ();

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        Ok(parts
            .headers
            .get("HX-Request")
            .map(|_| Htmx::Enabled)
            .unwrap_or(Htmx::Disabled))
    }
}
