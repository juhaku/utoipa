use std::borrow::Cow;

use actix_web::{dev::HttpServiceFactory, guard::Get, web, HttpResponse, Resource, Responder};
use rust_embed::RustEmbed;

use crate::openapi::OpenApi;

#[derive(RustEmbed)]
#[folder = "target/swagger-ui-3.52.5/dist/"]
pub struct SwaggerUiDist;

#[non_exhaustive]
#[derive(Clone)]
pub struct SwaggerUi {
    path: Cow<'static, str>,
    urls: Vec<(Url<'static>, OpenApi)>,
}

impl SwaggerUi {
    pub fn new<P: Into<Cow<'static, str>>>(path: P) -> Self {
        Self {
            path: path.into(),
            urls: Vec::new(),
        }
    }

    pub fn with_url<U: Into<Url<'static>>>(mut self, url: U, openapi: OpenApi) -> Self {
        self.urls.push((url.into(), openapi));

        self
    }

    pub fn with_urls(mut self, urls: Vec<(Url<'static>, OpenApi)>) -> Self {
        self.urls = urls;

        self
    }
}

#[cfg(feature = "actix-web")]
impl HttpServiceFactory for SwaggerUi {
    fn register(self, config: &mut actix_web::dev::AppService) {
        let urls = self
            .urls
            .into_iter()
            .map(|url| {
                register_api_doc_url_resource(&url, config);
                url.0
            })
            .collect::<Vec<_>>();

        let swagger_resource = Resource::new(self.path.as_ref())
            .guard(Get())
            .data(urls)
            .to(serve_swagger_ui);

        HttpServiceFactory::register(swagger_resource, config);
    }
}

#[cfg(feature = "actix-web")]
fn register_api_doc_url_resource(url: &(Url, OpenApi), config: &mut actix_web::dev::AppService) {
    pub async fn get_api_doc(api_doc: web::Data<OpenApi>) -> impl Responder {
        log::trace!("Get api doc:\n{}", api_doc.to_pretty_json().unwrap());

        HttpResponse::Ok().json(api_doc.as_ref())
    }

    let url_resource = Resource::new(url.0.url.as_ref())
        .guard(Get())
        .data(url.1.clone())
        .to(get_api_doc);
    HttpServiceFactory::register(url_resource, config);
}

#[non_exhaustive]
#[derive(Default, Clone)]
pub struct Url<'a> {
    name: Cow<'a, str>,
    url: Cow<'a, str>,
    primary: bool,
}

impl<'a> Url<'a> {
    pub fn new(name: &'a str, url: &'a str) -> Self {
        Self {
            name: Cow::Borrowed(name),
            url: Cow::Borrowed(url),
            ..Default::default()
        }
    }

    pub fn with_primary(name: &'a str, url: &'a str, primary: bool) -> Self {
        Self {
            name: Cow::Borrowed(name),
            url: Cow::Borrowed(url),
            primary,
        }
    }
}

impl<'a> From<&'a str> for Url<'a> {
    fn from(url: &'a str) -> Self {
        Self {
            url: Cow::Borrowed(url),
            ..Default::default()
        }
    }
}

impl From<String> for Url<'_> {
    fn from(url: String) -> Self {
        Self {
            url: Cow::Owned(url),
            ..Default::default()
        }
    }
}

#[cfg(feature = "actix-web")]
async fn serve_swagger_ui(
    web::Path(mut part): web::Path<String>,
    data: web::Data<Vec<Url<'_>>>,
) -> HttpResponse {
    use crate::error::Error;

    log::debug!("Get swagger resource: {}", &part);

    if part.is_empty() || part == "/" {
        part = "index.html".to_string()
    }

    if let Some(file) = SwaggerUiDist::get(&part) {
        let mut bytes = file.data.into_owned();

        if part == "index.html" {
            let mut index = match String::from_utf8(bytes.to_vec()).map_err(Error::FromUtf8) {
                Ok(index) => index,
                Err(error) => return HttpResponse::InternalServerError().body(error.to_string()),
            };

            if data.len() > 1 {
                let mut urls = String::from("urls: [");
                data.as_ref().iter().for_each(|url| {
                    urls.push_str(&format!(
                        "{{name: \"{}\", url: \"{}\"}},",
                        if url.name.is_empty() {
                            &url.url
                        } else {
                            &url.name
                        },
                        url.url
                    ));
                });
                urls.push(']');
                if let Some(primary) = data.as_ref().iter().find(|url| url.primary) {
                    urls.push_str(&format!(", \"urls.primaryName\": \"{}\"", primary.name));
                }
                index = index.replace(r"{{urls}}", &urls);
            } else if let Some(url) = data.first() {
                index = index.replace(r"{{urls}}", &format!("url: \"{}\"", url.url));
            }

            bytes = index.as_bytes().to_vec();
        };

        HttpResponse::Ok()
            .content_type(
                mime_guess::from_path(&part)
                    .first_or_octet_stream()
                    .to_string(),
            )
            .body(bytes)
    } else {
        HttpResponse::NotFound().finish()
    }
}
