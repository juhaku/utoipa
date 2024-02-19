#![cfg(feature = "rocket")]

use rocket::http::Method;
use rocket::response::content::RawHtml;
use rocket::route::{Handler, Outcome};
use rocket::{Data, Request, Route};

use crate::{Redoc, Spec};

impl<S: Spec> From<Redoc<S>> for Vec<Route> {
    fn from(value: Redoc<S>) -> Self {
        vec![Route::new(
            Method::Get,
            value.url.as_ref(),
            RedocHandler(value.to_html()),
        )]
    }
}

#[derive(Clone)]
struct RedocHandler(String);

#[rocket::async_trait]
impl Handler for RedocHandler {
    async fn handle<'r>(&self, request: &'r Request<'_>, _: Data<'r>) -> Outcome<'r> {
        Outcome::from(request, RawHtml(self.0.clone()))
    }
}
