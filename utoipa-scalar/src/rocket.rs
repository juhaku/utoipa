#![cfg(feature = "rocket")]

use rocket::http::Method;
use rocket::response::content::RawHtml;
use rocket::route::{Handler, Outcome};
use rocket::{Data, Request, Route};

use crate::{Scalar, Spec};

impl<S: Spec> From<Scalar<S>> for Vec<Route> {
    fn from(value: Scalar<S>) -> Self {
        vec![Route::new(
            Method::Get,
            value.url.as_ref(),
            ScalarHandler(value.to_html()),
        )]
    }
}

#[derive(Clone)]
struct ScalarHandler(String);

#[rocket::async_trait]
impl Handler for ScalarHandler {
    async fn handle<'r>(&self, request: &'r Request<'_>, _: Data<'r>) -> Outcome<'r> {
        Outcome::from(request, RawHtml(self.0.clone()))
    }
}
