use std::{borrow::Cow, ops::Deref};

use actix_web::{web, HttpResponse, Resource};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/swagger_ui/dist/"]
pub struct SwaggerUi;

pub struct SwaggerService {
    path: Cow<'static, str>,
    // TODO need to know all possible api urls
    url: Cow<'static, str>,
    // TODO need to know application json
}

#[cfg(feature = "actix-web")]
impl SwaggerService {
    pub fn new<S: Into<Cow<'static, str>>>(path: S, url: S) -> Self {
        Self {
            path: path.into(),
            url: url.into(),
        }
    }

    pub fn resource(&self) -> Resource {
        web::resource(self.path.deref())
            .data(self.url.to_string())
            .route(web::get().to(serve_swagger_ui))
    }
}

#[cfg(feature = "actix-web")]
async fn serve_swagger_ui(
    web::Path(mut part): web::Path<String>,
    data: web::Data<String>,
) -> HttpResponse {
    log::debug!("Get swagger resource: {}", &part);

    if part.is_empty() || part == "/" {
        part = "index.html".to_string()
    }

    log::debug!("Replace urls: {:?}", data.as_ref());

    // TODO replace urls with correct urls from index
    // TODO provide the api doc and serve info

    match SwaggerUi::get(&part) {
        Some(file) => {
            let bytes = file.data.into_owned();

            HttpResponse::Ok()
                .content_type(
                    mime_guess::from_path(&part)
                        .first_or_octet_stream()
                        .to_string(),
                )
                .body(bytes)
        }
        None => HttpResponse::NotFound().finish(),
    }
}
