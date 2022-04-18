//! This crate implements necessary boiler plate code to serve Swagger UI via web server. It
//! works as a bridge for serving the OpenAPI documetation created with [`utoipa`][utoipa] libarary in the
//! Swagger UI.
//!
//! [utoipa]: <https://docs.rs/utoipa/>
//!
//! **Currently implemented boiler plate for:**
//!
//! * **actix-web**
//! * **rocket** `version >=0.5.0-rc.1`
//!
//! Serving Swagger UI is framework independant thus this crate also supports serving the Swagger UI with
//! other frameworks as well. With other frameworks there is bit more manual implementation to be done. See
//! more details at [`serve`] or [`examples`][examples].
//!
//! [examples]: <https://github.com/juhaku/utoipa/tree/master/examples>
//!
//! # Features
//!
//! * **actix-web** Enables actix-web integration with pre-configured SwaggerUI service factory allowing
//!   users to use the Swagger UI without a hazzle.
//! * **rocket** Enables rocket integration with with pre-configured routes for serving the Swagger UI
//!   and api doc without a hazzle.
//!
//! # Install
//!
//! Use only the raw types without any boiler plate implementation.
//! ```text
//! [dependencies]
//! utoipa-swagger-ui = "0.2.0"
//!
//! ```
//! Enable actix-web framework with Swagger UI you could define the dependency as follows.
//! ```text
//! [dependencies]
//! utoipa-swagger-ui = { version = "0.2.0", features = ["actix-web"] }
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
//! # #[openapi(handlers())]
//! # struct ApiDoc;
//! HttpServer::new(move || {
//!         App::new()
//!             .service(
//!                 SwaggerUi::new("/swagger-ui/{_:.*}")
//!                     .url("/api-doc/openapi.json", ApiDoc::openapi()),
//!             )
//!     })
//!     .bind((Ipv4Addr::UNSPECIFIED, 8989)).unwrap()
//!     .run();
//! ```
//!
//! Serve Swagger UI with api doc via rocket [^rocket]
//! ```no_run
//! # use rocket::{Build, Rocket};
//! # use utoipa_swagger_ui::SwaggerUi;
//! # use utoipa::OpenApi;
//! #[rocket::launch]
//! fn rocket() -> Rocket<Build> {
//! #
//! #     #[derive(OpenApi)]
//! #     #[openapi()]
//! #     struct ApiDoc;
//! #
//!     rocket::build()
//!         .mount(
//!             "/",
//!             SwaggerUi::new("/swagger-ui/<_..>").url("/api-doc/openapi.json", ApiDoc::openapi()),
//!         )
//! }
//! ```
//!
//! [^actix]: **actix-web** feature need to be enabled.
//!
//! [^rocket]: **rocket** feature need to be enabled.
use std::{borrow::Cow, error::Error, sync::Arc};

#[cfg(feature = "actix-web")]
use actix_web::{
    dev::HttpServiceFactory, guard::Get, web, web::Data, HttpResponse, Resource,
    Responder as ActixResponder,
};

#[cfg(feature = "rocket")]
use std::io::Cursor;

#[cfg(feature = "rocket")]
use rocket::{
    http::{Header, Status},
    response::{
        status::{self, NotFound},
        Responder as RocketResponder,
    },
    route::{Handler, Outcome},
    serde::json::Json,
    Data as RocketData, Request, Response, Route,
};

use rust_embed::RustEmbed;
#[cfg(any(feature = "actix-web", feature = "rocket"))]
use utoipa::openapi::OpenApi;

#[derive(RustEmbed)]
#[folder = "$UTOIPA_SWAGGER_DIR/$UTOIPA_SWAGGER_UI_VERSION/dist/"]
struct SwaggerUiDist;

/// Entry point for serving Swagger UI and api docs in application. It uses provides
/// builder style chainable configuration methods for configuring api doc urls. **In actix-web only** [^actix]
///
/// [^actix]: **actix-web** feature need to be enabled.
#[non_exhaustive]
#[derive(Clone)]
#[cfg(any(feature = "actix-web", feature = "rocket"))]
pub struct SwaggerUi {
    path: Cow<'static, str>,
    urls: Vec<(Url<'static>, OpenApi)>,
}

#[cfg(any(feature = "actix-web", feature = "rocket"))]
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
    /// Calling this again will add another url to the Swagger UI.
    ///
    /// # Examples
    ///
    /// Expose manually created OpenAPI doc.
    /// ```rust
    /// # use utoipa_swagger_ui::SwaggerUi;
    /// let swagger = SwaggerUi::new("/swagger-ui/{_:.*}")
    ///     .url("/api-doc/openapi.json", utoipa::openapi::OpenApi::new(
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
    /// # #[openapi(handlers())]
    /// # struct ApiDoc;
    /// let swagger = SwaggerUi::new("/swagger-ui/{_:.*}")
    ///     .url("/api-doc/openapi.json", ApiDoc::openapi());
    /// ```
    pub fn url<U: Into<Url<'static>>>(mut self, url: U, openapi: OpenApi) -> Self {
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
    /// # #[openapi(handlers())]
    /// # struct ApiDoc;
    /// # #[derive(OpenApi)]
    /// # #[openapi(handlers())]
    /// # struct ApiDoc2;
    /// let swagger = SwaggerUi::new("/swagger-ui/{_:.*}")
    ///     .urls(
    ///       vec![
    ///          (Url::with_primary("api doc 1", "/api-doc/openapi.json", true), ApiDoc::openapi()),
    ///          (Url::new("api doc 2", "/api-doc/openapi2.json"), ApiDoc2::openapi())
    ///     ]
    /// );
    /// ```
    pub fn urls(mut self, urls: Vec<(Url<'static>, OpenApi)>) -> Self {
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
                let (url, openapi) = url;
                register_api_doc_url_resource(url.url.as_ref(), openapi, config);
                url
            })
            .collect::<Vec<_>>();

        let swagger_resource = Resource::new(self.path.as_ref())
            .guard(Get())
            .app_data(Data::new(Config::new(urls)))
            .to(serve_swagger_ui);

        HttpServiceFactory::register(swagger_resource, config);
    }
}

#[cfg(feature = "actix-web")]
fn register_api_doc_url_resource(url: &str, api: OpenApi, config: &mut actix_web::dev::AppService) {
    pub async fn get_api_doc(api_doc: web::Data<OpenApi>) -> impl ActixResponder {
        HttpResponse::Ok().json(api_doc.as_ref())
    }

    let url_resource = Resource::new(url)
        .guard(Get())
        .app_data(Data::new(api))
        .to(get_api_doc);
    HttpServiceFactory::register(url_resource, config);
}

#[cfg(feature = "rocket")]
impl From<SwaggerUi> for Vec<Route> {
    fn from(swagger_ui: SwaggerUi) -> Self {
        let mut routes = Vec::<Route>::with_capacity(swagger_ui.urls.len() + 1);
        let mut api_docs = Vec::<Route>::with_capacity(swagger_ui.urls.len());

        let urls = swagger_ui.urls.into_iter().map(|(url, openapi)| {
            api_docs.push(Route::new(
                rocket::http::Method::Get,
                url.url.as_ref(),
                ServeApiDoc(openapi),
            ));
            url
        });

        routes.push(Route::new(
            rocket::http::Method::Get,
            swagger_ui.path.as_ref(),
            ServeSwagger(swagger_ui.path.clone(), Arc::new(Config::new(urls))),
        ));
        routes.extend(api_docs);

        routes
    }
}

#[cfg(feature = "rocket")]
#[derive(Clone)]
struct ServeApiDoc(utoipa::openapi::OpenApi);

#[cfg(feature = "rocket")]
#[rocket::async_trait]
impl Handler for ServeApiDoc {
    async fn handle<'r>(&self, request: &'r Request<'_>, _: RocketData<'r>) -> Outcome<'r> {
        Outcome::from(request, Json(self.0.clone()))
    }
}

#[cfg(feature = "rocket")]
#[derive(Clone)]
struct ServeSwagger(Cow<'static, str>, Arc<Config<'static>>);

#[cfg(feature = "rocket")]
#[rocket::async_trait]
impl Handler for ServeSwagger {
    async fn handle<'r>(&self, request: &'r Request<'_>, _: RocketData<'r>) -> Outcome<'r> {
        let mut path = self.0.as_ref();
        if let Some(index) = self.0.find('<') {
            path = &path[..index];
        }

        match serve(&request.uri().path().as_str()[path.len()..], self.1.clone()) {
            Ok(swagger_file) => swagger_file
                .map(|file| Outcome::from(request, file))
                .unwrap_or_else(|| Outcome::from(request, NotFound("Swagger UI file not found"))),
            Err(error) => Outcome::from(
                request,
                status::Custom(Status::InternalServerError, error.to_string()),
            ),
        }
    }
}

#[cfg(feature = "rocket")]
impl<'r, 'o: 'r> RocketResponder<'r, 'o> for SwaggerFile<'o> {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'o> {
        rocket::response::Result::Ok(
            Response::build()
                .header(Header::new("Content-Type", self.content_type))
                .sized_body(self.bytes.len(), Cursor::new(self.bytes.to_vec()))
                .status(Status::Ok)
                .finalize(),
        )
    }
}

/// Rust type for Swagger UI url configuration object.
#[non_exhaustive]
#[derive(Default, Clone, Debug)]
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

    fn to_json_object_string(&self) -> String {
        format!(
            r#"{{name: "{}", url: "{}"}}"#,
            if self.name.is_empty() {
                &self.url
            } else {
                &self.name
            },
            self.url
        )
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

impl<'a> From<Cow<'static, str>> for Url<'a> {
    fn from(url: Cow<'static, str>) -> Self {
        Self {
            url,
            ..Default::default()
        }
    }
}

#[cfg(feature = "actix-web")]
async fn serve_swagger_ui(path: web::Path<String>, data: web::Data<Config<'_>>) -> HttpResponse {
    match serve(&*path.into_inner(), data.into_inner()) {
        Ok(swagger_file) => swagger_file
            .map(|file| {
                HttpResponse::Ok()
                    .content_type(file.content_type)
                    .body(file.bytes.to_vec())
            })
            .unwrap_or_else(|| HttpResponse::NotFound().finish()),
        Err(error) => HttpResponse::InternalServerError().body(error.to_string()),
    }
}

/// Object used to alter Swagger UI settings.
///
/// # Examples
///
/// Simple case is to create config directly from url that points to the api doc json.
/// ```rust
/// # use utoipa_swagger_ui::Config;
/// let config = Config::from("/api-doc.json");
/// ```
///
/// If there is multiple api docs to serve config can be also directly created with [`Config::new`]
/// ```rust
/// # use utoipa_swagger_ui::Config;
/// let config = Config::new(["/api-doc/openapi1.json", "/api-doc/openapi2.json"]);
/// ```
///
/// Or same as above but more verbose syntax.
/// ```rust
/// # use utoipa_swagger_ui::{Config, Url};
/// let config = Config::new([
///     Url::new("api1", "/api-doc/openapi1.json"),
///     Url::new("api2", "/api-doc/openapi2.json")
/// ]);
/// ```
#[non_exhaustive]
#[derive(Default, Clone)]
pub struct Config<'a> {
    /// [`Url`]s the Swagger UI is serving.
    urls: Vec<Url<'a>>,
}

impl<'a> Config<'a> {
    /// Constructs a new [`Config`] from [`Iterator`] of [`Url`]s.
    ///
    /// # Examples
    /// Create new config with 2 api doc urls.
    /// ```rust
    /// # use utoipa_swagger_ui::Config;
    /// let config = Config::new(["/api-doc/openapi1.json", "/api-doc/openapi2.json"]);
    /// ```
    pub fn new<I: IntoIterator<Item = U>, U: Into<Url<'a>>>(urls: I) -> Self {
        Self {
            urls: urls.into_iter().map(|url| url.into()).collect(),
        }
    }
}

impl<'a> From<&'a str> for Config<'a> {
    fn from(s: &'a str) -> Self {
        Self {
            urls: vec![Url::from(s)],
        }
    }
}

impl From<String> for Config<'_> {
    fn from(s: String) -> Self {
        Self {
            urls: vec![Url::from(s)],
        }
    }
}

/// Represents servealbe file of Swagger UI. This is used together with [`serve`] function
/// to serve Swagger UI files via web server.
#[non_exhaustive]
pub struct SwaggerFile<'a> {
    /// Content of the file as [`Cow`] [`slice`] of bytes.
    pub bytes: Cow<'a, [u8]>,
    /// Content type of the file e.g `"text/xml"`.
    pub content_type: String,
}

/// User friendly way to serve Swagger UI and its content via web server.
///
/// * **path** Should be the relative path to Swagger UI resource within the web server.
/// * **config** Swagger [`Config`] to use for the Swagger UI. Currently supported configuration
///   options are managing [`Url`]s.
///
/// Typpically this function is implemented _**within**_ handler what handles _**GET**_ operations related to the
/// Swagger UI. Handler itself must match to user defined path that points to the root of the Swagger UI and
/// matches everything relatively from the root of the Swagger UI. The relative path from root of the Swagger UI
/// must be taken to `tail` path variable which is used to serve [`SwaggerFile`]s. If Swagger UI
/// is served from path `/swagger-ui/` then the `tail` is everything under the `/swagger-ui/` prefix.
///
/// _There are also implementations in [examples of utoipa repoistory][examples]._
///
/// [examples]: https://github.com/juhaku/utoipa/tree/master/examples
///
/// # Examples
///
/// Reference implementation with `actix-web`.
/// ```rust
/// # use actix_web::HttpResponse;
/// # use std::sync::Arc;
/// # use utoipa_swagger_ui::Config;
/// // The config should be created in main function or in initialization before
/// // creation of the handler which will handle serving the Swagger UI.
/// let config = Arc::new(Config::from("/api-doc.json"));
/// // This "/" is for demostrative purposes only. The actual path should point to
/// // file within Swagger UI. In real implementation this is the `tail` path from root of the
/// // Swagger UI to the file served.
/// let path = "/";
///
/// match utoipa_swagger_ui::serve(path, config) {
///     Ok(swagger_file) => swagger_file
///         .map(|file| {
///             HttpResponse::Ok()
///                 .content_type(file.content_type)
///                 .body(file.bytes.to_vec())
///         })
///         .unwrap_or_else(|| HttpResponse::NotFound().finish()),
///     Err(error) => HttpResponse::InternalServerError().body(error.to_string()),
/// };
/// ```
pub fn serve<'a>(
    path: &str,
    config: Arc<Config<'a>>,
) -> Result<Option<SwaggerFile<'a>>, Box<dyn Error>> {
    let mut file_path = path;

    if file_path.is_empty() || file_path == "/" {
        file_path = "index.html";
    }

    if let Some(file) = SwaggerUiDist::get(file_path) {
        let mut bytes = file.data;

        if file_path == "swagger-initializer.js" {
            let mut file = match String::from_utf8(bytes.to_vec()) {
                Ok(index) => index,
                Err(error) => return Err(Box::new(error)),
            };
            file = format_swagger_config_urls(&mut config.urls.iter(), file);

            bytes = Cow::Owned(file.as_bytes().to_vec())
        };

        Ok(Some(SwaggerFile {
            bytes,
            content_type: mime_guess::from_path(&file_path)
                .first_or_octet_stream()
                .to_string(),
        }))
    } else {
        Ok(None)
    }
}

#[inline]
fn format_swagger_config_urls<'a, U: ExactSizeIterator<Item = &'a Url<'a>>>(
    urls: &mut U,
    file: String,
) -> String {
    if urls.len() > 1 {
        let mut primary = None::<Cow<'a, str>>;
        let mut urls_string = format!(
            "urls: [{}],",
            &urls
                .inspect(|url| if url.primary {
                    primary = Some(Cow::Borrowed(url.name.as_ref()))
                })
                .map(Url::to_json_object_string)
                .collect::<Vec<_>>()
                .join(",")
        );

        if let Some(primary) = primary {
            urls_string.push_str(&format!(r#""urls.primaryName": "{}","#, primary));
        }
        file.replace(r"{{urls}},", &urls_string)
    } else if let Some(url) = urls.next() {
        file.replace(r"{{urls}}", &format!(r#"url: "{}""#, url.url))
    } else {
        file
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_CONTENT: &str = r###""window.ui = SwaggerUIBundle({
    {{urls}},
    dom_id: '#swagger-ui',
    deepLinking: true,
    presets: [
      SwaggerUIBundle.presets.apis,
      SwaggerUIStandalonePreset
    ],
    plugins: [
      SwaggerUIBundle.plugins.DownloadUrl
    ],
    layout: "StandaloneLayout"
  });""###;

    #[test]
    fn format_swagger_config_urls_with_one_url() {
        let config = Config::from("/api-doc.json");
        let file =
            super::format_swagger_config_urls(&mut config.urls.iter(), TEST_CONTENT.to_string());

        assert!(
            file.contains(r#"url: "/api-doc.json","#),
            "expected file to contain {}",
            r#"url: "/api-doc.json","#
        )
    }

    #[test]
    fn format_swagger_config_urls_multiple() {
        let config = Config::new(["/api-doc.json", "/api-doc2.json"]);
        let file =
            super::format_swagger_config_urls(&mut config.urls.iter(), TEST_CONTENT.to_string());

        assert!(
            file.contains(r#"urls: [{name: "/api-doc.json", url: "/api-doc.json"},{name: "/api-doc2.json", url: "/api-doc2.json"}],"#),
            "expected file to contain {}",
            r#"urls: [{name: "/api-doc.json", url: "/api-doc.json"}, {name: "/api-doc2.json", url: "/api-doc2.json"}],"#
        )
    }
    #[test]
    fn format_swagger_config_urls_with_primary() {
        let config = Config::new([
            Url::new("api1", "/api-doc.json"),
            Url::with_primary("api2", "/api-doc2.json", true),
        ]);
        let file =
            super::format_swagger_config_urls(&mut config.urls.iter(), TEST_CONTENT.to_string());

        assert!(
            file.contains(r#"urls: [{name: "api1", url: "/api-doc.json"},{name: "api2", url: "/api-doc2.json"}],"#),
            "expected file to contain {}",
            r#"urls: [{name: "api1", url: "/api-doc.json"}, {name: "api2", url: "/api-doc2.json"}],"#
        );
        assert!(
            file.contains(r#""urls.primaryName": "api2","#),
            "expected file to contain {}",
            r#""urls.primaryName": "api2","#
        )
    }
}
