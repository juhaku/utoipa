use std::{borrow::Cow, ops::Deref};

use actix_web::{dev::HttpServiceFactory, guard::Get, web, HttpResponse, Resource, Responder};
use rust_embed::RustEmbed;

use crate::openapi::OpenApi;

#[derive(RustEmbed)]
#[folder = "target/swagger-ui-3.52.5/dist/"]
pub struct SwaggerUiDist;

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

#[non_exhaustive]
// #[derive(Clone)]
pub struct SwaggerUi {
    path: String,
    url: Option<(Url, OpenApi)>,
    urls: Option<Vec<(Url, OpenApi)>>,
}

impl SwaggerUi {
    pub fn new<S: Into<String>>(path: S) -> Self {
        Self {
            path: path.into(),
            url: None,
            urls: None,
        }
    }

    pub fn with_url<U: Into<Url>>(mut self, url: U, openapi: OpenApi) -> Self {
        self.url = Some((url.into(), openapi));

        self
    }

    pub fn with_urls<U: 'static + Into<Url>>(mut self, urls: &'static [(U, OpenApi)]) -> Self
    where
        U: 'static + Into<Url>,
        Url: From<&'static U>,
    {
        let u = urls
            .iter()
            .map(|(url, api)| (Into::<Url>::into(url), api.clone()))
            .collect::<Vec<_>>();
        self.urls = Some(u);

        self
    }
}

#[cfg(feature = "actix-web")]
impl HttpServiceFactory for SwaggerUi {
    fn register(self, config: &mut actix_web::dev::AppService) {
        let urls = self
            .url
            .map(|url| {
                register_api_doc_url_resource(&url, config);
                vec![url.0]
            })
            .or_else(|| {
                self.urls.map(|slice| {
                    slice
                        .iter()
                        .map(|url| {
                            register_api_doc_url_resource(url, config);
                            url.0.clone()
                        })
                        .collect()
                })
            })
            .unwrap_or_default();

        let swagger_resource = Resource::new(self.path)
            .guard(Get())
            .data(urls)
            .to(serve_swagger_ui);

        HttpServiceFactory::register(swagger_resource, config);
    }
}

#[cfg(feature = "actix-web")]
fn register_api_doc_url_resource(url: &(Url, OpenApi), config: &mut actix_web::dev::AppService) {
    pub async fn get_api_doc(api_doc: web::Data<OpenApi>) -> impl Responder {
        log::debug!("Get api doc:\n{}", api_doc.to_pretty_json().unwrap());

        HttpResponse::Ok().json(api_doc.as_ref())
    }

    let url_resource = Resource::new(&url.0.url)
        .guard(Get())
        .data(url.1.clone())
        .to(get_api_doc);
    HttpServiceFactory::register(url_resource, config);
}

#[non_exhaustive]
#[derive(Default, Clone)]
pub struct Url {
    name: String,
    url: String,
    primary: bool,
}

impl Url {
    pub fn new<S>(name: S, url: S, primary: bool) -> Self
    where
        S: Into<String>,
    {
        Self {
            name: name.into(),
            url: url.into(),
            primary,
        }
    }

    // fn with_url<S: 'a + AsRef<&'a str>>(mut self, url: S) -> Self {
    //     self.url = Cow::Borrowed(url.as_ref());

    //     self
    // }

    // fn with_name<S: 'a + AsRef<&'a str>>(mut self, name: S) -> Self {
    //     self.name = Cow::Borrowed(name.as_ref());

    //     self
    // }

    // fn with_primary(mut self, primary: bool) -> Self {
    //     self.primary = primary;

    //     self
    // }
}

impl From<&'static str> for Url {
    fn from(url: &'static str) -> Self {
        Self {
            url: String::from(url),
            ..Default::default()
        }
    }
}

// impl From<str> for Url {
//     fn from(url: str) -> Self {
//         Self {
//             url: Cow::Owned(url),
//             ..Default::default()
//         }
//     }
// }

impl From<String> for Url {
    fn from(url: String) -> Self {
        Self {
            url,
            ..Default::default()
        }
    }
}

#[cfg(feature = "actix-web")]
async fn serve_swagger_ui(
    web::Path(mut part): web::Path<String>,
    data: web::Data<Vec<Url>>,
) -> HttpResponse {
    use crate::error::Error;

    log::debug!("Get swagger resource: {}", &part);

    if part.is_empty() || part == "/" {
        part = "index.html".to_string()
    }

    // log::debug!("Replace urls: {:?}", data.as_ref());

    // TODO replace urls with correct urls from index
    // TODO provide the api doc and serve info

    if let Some(file) = SwaggerUiDist::get(&part) {
        let mut bytes = file.data.into_owned();

        if part == "index.html" {
            // TODO replace the url wihtin the content
            let mut index = match String::from_utf8(bytes.to_vec()).map_err(Error::FromUtf8) {
                Ok(index) => index,
                Err(error) => return HttpResponse::InternalServerError().body(error.to_string()),
            };

            // url: "https://petstore.swagger.io/v2/swagger.json",
            if data.len() > 1 {
                // TODO multiple
                let mut urls = String::from("urls: [");
                data.as_ref().iter().for_each(|url| {
                    urls.push_str(&format!("{{name: {}, url: {}}},", url.name, url.url));
                });
                urls.push(']');
                if let Some(primary) = data.as_ref().iter().find(|url| url.primary) {
                    urls.push_str(&format!(", urls.primaryName: {}", primary.name));
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
