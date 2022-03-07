//! This crate implements necessary boiler plate code to serve Swagger UI via web server. It
//! works as a bridge for serving the OpenAPI documetation created with [`utoipa`][utoipa] libarary in the
//! Swagger UI.
//!
//! [utoipa]: <https://docs.rs/utoipa/>
//!
//! **Currently supported frameworks:**
//!
//! * **actix-web**
//!
//! Serving Swagger UI is framework independant thus [`SwaggerUi`] and [`Url`] of this create
//! could be used similarly to serve the Swagger UI in other frameworks as well.
//!
//! # Features
//!
//! * **actix-web** Enables actix-web integration with pre-configured SwaggerUI service factory allowing
//!   users to use the Swagger UI without a hazzle.
//!
//! # Install
//!
//! Use only the raw types without any boiler plate implementation.
//! ```text
//! [dependencies]
//! utoipa-swagger-ui = "0.1.0.rc1"
//!
//! ```
//! Enable actix-web framework with Swagger UI you could define the dependency as follows.
//! ```text
//! [dependencies]
//! utoipa-swagger-ui = { version = "0.1.0.rc1", features = ["actix-web"] }
//! ```
//!
//! **Note!** Also remember that you already have defined `utoipa` dependency in your `Cargo.toml`
//!
//! # Examples
//!
//! Serve Swagger UI with api doc via actix-web. [^actix]
//! ```no_run
//! # use actix_web::{App, HttpServer};
//! # use utoipa_swagger_ui::SwaggerUi;
//! # use utoipa::OpenApi;
//! # use std::net::Ipv4Addr;
//! # #[derive(OpenApi)]
//! # #[openapi(handlers = [])]
//! # struct ApiDoc;
//! HttpServer::new(move || {
//!         App::new()
//!             .service(
//!                 SwaggerUi::new("/swagger-ui/{_:.*}")
//!                     .with_url("/api-doc/openapi.json", ApiDoc::openapi()),
//!             )
//!     })
//!     .bind(format!("{}:{}", Ipv4Addr::UNSPECIFIED, 8989)).unwrap()
//!     .run();
//! ```
//! [^actix]: **actix-web** feature need to be enabled.
use std::borrow::Cow;

#[cfg(feature = "actix-web")]
use actix_web::{dev::HttpServiceFactory, guard::Get, web, HttpResponse, Resource, Responder};

use rust_embed::RustEmbed;
use utoipa::openapi::OpenApi;

#[doc(hidden)]
#[derive(RustEmbed)]
#[folder = "$UTOIPA_SWAGGER_DIR/$UTOIPA_SWAGGER_UI_VERSION/dist/"]
pub struct SwaggerUiDist;

/// Entry point for serving Swagger UI and api docs in application.
#[non_exhaustive]
#[derive(Clone)]
pub struct SwaggerUi {
    path: Cow<'static, str>,
    urls: Vec<(Url<'static>, OpenApi)>,
}

impl SwaggerUi {
    /// Create a new [`SwaggerUi`] for given path.
    ///
    /// Path argument will expose the Swagger UI to the user and should be something that
    /// the underlying application framework / library supports.
    ///
    /// # Examples
    ///
    /// Exposes Swagger UI using path `/swagger-ui` using actix-web supported syntax.
    ///
    /// ```rust
    /// # use utoipa_swagger_ui::SwaggerUi;
    /// let swagger = SwaggerUi::new("/swagger-ui/{_:.*}");
    /// ```
    pub fn new<P: Into<Cow<'static, str>>>(path: P) -> Self {
        Self {
            path: path.into(),
            urls: Vec::new(),
        }
    }

    /// Add api doc [`Url`] into [`SwaggerUi`].
    ///
    /// Method takes two arguments where first one is path which exposes the [`OpenApi`] to the user.
    /// Second argument is the actual Rust implementation of the OpenAPI doc which is being exposed.
    ///
    /// # Examples
    ///
    /// Expose manually created OpenAPI doc.
    /// ```rust
    /// # use utoipa_swagger_ui::SwaggerUi;
    /// let swagger = SwaggerUi::new("/swagger-ui/{_:.*}")
    ///     .with_url("/api-doc/openapi.json", utoipa::openapi::OpenApi::new(
    ///        utoipa::openapi::Info::new("my application", "0.1.0"),
    ///        utoipa::openapi::Paths::new(),
    /// ));
    /// ```
    ///
    /// Expose derived OpenAPI doc.
    /// ```rust
    /// # use utoipa_swagger_ui::SwaggerUi;
    /// # use utoipa::OpenApi;
    /// # #[derive(OpenApi)]
    /// # #[openapi(handlers = [])]
    /// # struct ApiDoc;
    /// let swagger = SwaggerUi::new("/swagger-ui/{_:.*}")
    ///     .with_url("/api-doc/openapi.json", ApiDoc::openapi());
    /// ```
    pub fn with_url<U: Into<Url<'static>>>(mut self, url: U, openapi: OpenApi) -> Self {
        self.urls.push((url.into(), openapi));

        self
    }

    /// Add multiple [`Url`]s to Swagger UI.
    ///
    /// Takes one [`Vec`] argument containing tuples of [`Url`] and [`OpenApi`].
    ///
    /// Situations where this comes handy is when there is a need or wish to seprate different parts
    /// of the api to separate api docs.
    ///
    /// # Examples
    ///
    /// Expose multiple api docs via Swagger UI.
    /// ```rust
    /// # use utoipa_swagger_ui::{SwaggerUi, Url};
    /// # use utoipa::OpenApi;
    /// # #[derive(OpenApi)]
    /// # #[openapi(handlers = [])]
    /// # struct ApiDoc;
    /// # #[derive(OpenApi)]
    /// # #[openapi(handlers = [])]
    /// # struct ApiDoc2;
    /// let swagger = SwaggerUi::new("/swagger-ui/{_:.*}")
    ///     .with_urls(
    ///       vec![
    ///          (Url::with_primary("api doc 1", "/api-doc/openapi.json", true), ApiDoc::openapi()),
    ///          (Url::new("api doc 2", "/api-doc/openapi2.json"), ApiDoc2::openapi())
    ///     ]
    /// );
    /// ```
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
        HttpResponse::Ok().json(api_doc.as_ref())
    }

    let url_resource = Resource::new(url.0.url.as_ref())
        .guard(Get())
        .data(url.1.clone())
        .to(get_api_doc);
    HttpServiceFactory::register(url_resource, config);
}

/// Rust type for Swagger UI url configuration object.
#[non_exhaustive]
#[derive(Default, Clone)]
pub struct Url<'a> {
    name: Cow<'a, str>,
    url: Cow<'a, str>,
    primary: bool,
}

impl<'a> Url<'a> {
    /// Create new [`Url`].
    ///
    /// Name is shown in the select dropdown when there are multiple docs in Swagger UI.
    ///
    /// Url is path which exposes the OpenAPI doc.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use utoipa_swagger_ui::Url;
    /// let url = Url::new("My Api", "/api-doc/openapi.json");
    /// ```
    pub fn new(name: &'a str, url: &'a str) -> Self {
        Self {
            name: Cow::Borrowed(name),
            url: Cow::Borrowed(url),
            ..Default::default()
        }
    }

    /// Create new [`Url`] with primary flag.
    ///
    /// Primary flag allows users to override the default behaviour of the Swagger UI for selecting the primary
    /// doc to display. By default when there are multiple docs in Swagger UI the first one in the list
    /// will be the primary.
    ///
    /// Name is shown in the select dropdown when there are multiple docs in Swagger UI.
    ///
    /// Url is path which exposes the OpenAPI doc.
    ///
    /// # Examples
    ///
    /// Set "My Api" as primary.
    /// ```rust
    /// # use utoipa_swagger_ui::Url;
    /// let url = Url::with_primary("My Api", "/api-doc/openapi.json", true);
    /// ```
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
    if part.is_empty() || part == "/" {
        part = "index.html".to_string()
    }

    if let Some(file) = SwaggerUiDist::get(&part) {
        let mut bytes = file.data.into_owned();

        if part == "index.html" {
            let mut index = match String::from_utf8(bytes.to_vec()) {
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
