
use std::ops::Deref;
use actix_web::{FromRequest, HttpRequest, dev::Payload};
use crate::error::*;

#[derive(Debug, Copy, Clone)]
pub struct Extension<T>(T);


impl<T> Deref for Extension<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


impl<T: 'static> FromRequest for Extension<T> {
    type Error = Error;
    type Future = Result<Extension<T>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        req.extensions_mut().remove::<T>()
            .map(Extension)
            .ok_or_else(|| err!("No Extension configured, to configure use RequestHead::extensions()"))
    }
}

