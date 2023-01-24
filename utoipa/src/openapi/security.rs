//! Implements [OpenAPI Security Schema][security] types.
//!
//! Refer to [`SecurityScheme`] for usage and more details.
//!
//! [security]: https://spec.openapis.org/oas/latest.html#security-scheme-object
use std::{collections::BTreeMap, iter};

use serde::{Deserialize, Serialize};

use super::builder;

/// OpenAPI [security requirement][security] object.
///
/// Security requirement holds list of required [`SecurityScheme`] *names* and possible *scopes* required
/// to execute the operation. They can be defined in [`#[utoipa::path(...)]`][path] or in `#[openapi(...)]`
/// of [`OpenApi`][openapi].
///
/// Applying the security requirement to [`OpenApi`][openapi] will make it globally
/// available to all operations. When applied to specific [`#[utoipa::path(...)]`][path] will only
/// make the security requirements available for that operation. Only one of the requirements must be
/// satisfied.
///
/// [security]: https://spec.openapis.org/oas/latest.html#security-requirement-object
/// [path]: ../../attr.path.html
/// [openapi]: ../../derive.OpenApi.html
#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct SecurityRequirement {
    #[serde(flatten)]
    value: BTreeMap<String, Vec<String>>,
}

impl SecurityRequirement {
    /// Construct a new [`SecurityRequirement`]
    ///
    /// Accepts name for the security requirement which must match to the name of available [`SecurityScheme`].
    /// Second parameter is [`IntoIterator`] of [`Into<String>`] scopes needed by the [`SecurityRequirement`].
    /// Scopes must match to the ones defined in [`SecurityScheme`].
    ///
    /// # Examples
    ///
    /// Create new security requirement with scopes.
    /// ```rust
    /// # use utoipa::openapi::security::SecurityRequirement;
    /// SecurityRequirement::new("api_oauth2_flow", ["edit:items", "read:items"]);
    /// ```
    ///
    /// You can also create an empty security requirement with `Default::default()`.
    /// ```rust
    /// # use utoipa::openapi::security::SecurityRequirement;
    /// SecurityRequirement::default();
    /// ```
    pub fn new<N: Into<String>, S: IntoIterator<Item = I>, I: Into<String>>(
        name: N,
        scopes: S,
    ) -> Self {
        Self {
            value: BTreeMap::from_iter(iter::once_with(|| {
                (
                    Into::<String>::into(name),
                    scopes
                        .into_iter()
                        .map(|scope| Into::<String>::into(scope))
                        .collect::<Vec<_>>(),
                )
            })),
        }
    }
}

/// OpenAPI [security scheme][security] for path operations.
///
/// [security]: https://spec.openapis.org/oas/latest.html#security-scheme-object
///
/// # Examples
///
/// Create implicit oauth2 flow security schema for path operations.
/// ```rust
/// # use utoipa::openapi::security::{SecurityScheme, OAuth2, Implicit, Flow, Scopes};
/// SecurityScheme::OAuth2(
///     OAuth2::with_description([Flow::Implicit(
///         Implicit::new(
///             "https://localhost/auth/dialog",
///             Scopes::from_iter([
///                 ("edit:items", "edit my items"),
///                 ("read:items", "read my items")
///             ]),
///         ),
///     )], "my oauth2 flow")
/// );
/// ```
///
/// Create JWT header authentication.
/// ```rust
/// # use utoipa::openapi::security::{SecurityScheme, HttpAuthScheme, HttpBuilder};
/// SecurityScheme::Http(
///     HttpBuilder::new().scheme(HttpAuthScheme::Bearer).bearer_format("JWT").build()
/// );
/// ```
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum SecurityScheme {
    /// Oauth flow authentication.
    #[serde(rename = "oauth2")]
    OAuth2(OAuth2),
    /// Api key authentication sent in *`header`*, *`cookie`* or *`query`*.
    ApiKey(ApiKey),
    /// Http authentication such as *`bearer`* or *`basic`*.
    Http(Http),
    /// Open id connect url to discover OAuth2 configuration values.
    OpenIdConnect(OpenIdConnect),
    /// Authentication is done via client side certificate.
    ///
    /// OpenApi 3.1 type
    #[serde(rename = "mutualTLS")]
    MutualTls {
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
}

/// Api key authentication [`SecurityScheme`].
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(tag = "in", rename_all = "lowercase")]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum ApiKey {
    /// Create api key which is placed in HTTP header.
    Header(ApiKeyValue),
    /// Create api key which is placed in query parameters.
    Query(ApiKeyValue),
    /// Create api key which is placed in cookie value.
    Cookie(ApiKeyValue),
}

/// Value object for [`ApiKey`].
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ApiKeyValue {
    /// Name of the [`ApiKey`] parameter.
    pub name: String,

    /// Description of the the [`ApiKey`] [`SecurityScheme`]. Supports markdown syntax.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl ApiKeyValue {
    /// Constructs new api key value.
    ///
    /// # Examples
    ///
    /// Create new api key security schema with name `api_key`.
    /// ```rust
    /// # use utoipa::openapi::security::ApiKeyValue;
    /// let api_key = ApiKeyValue::new("api_key");
    /// ```
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            description: None,
        }
    }

    /// Construct a new api key with optional description supporting markdown syntax.
    ///
    /// # Examples
    ///
    /// Create new api key security schema with name `api_key` with description.
    /// ```rust
    /// # use utoipa::openapi::security::ApiKeyValue;
    /// let api_key = ApiKeyValue::with_description("api_key", "my api_key token");
    /// ```
    pub fn with_description<S: Into<String>>(name: S, description: S) -> Self {
        Self {
            name: name.into(),
            description: Some(description.into()),
        }
    }
}

builder! {
    HttpBuilder;

    /// Http authentication [`SecurityScheme`] builder.
    ///
    /// Methods can be chained to configure _bearer_format_ or to add _description_.
    #[non_exhaustive]
    #[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
    #[serde(rename_all = "camelCase")]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct Http {
        /// Http authorization scheme in HTTP `Authorization` header value.
        pub scheme: HttpAuthScheme,

        /// Optional hint to client how the bearer token is formatted. Valid only with [`HttpAuthScheme::Bearer`].
        #[serde(skip_serializing_if = "Option::is_none")]
        pub bearer_format: Option<String>,

        /// Optional description of [`Http`] [`SecurityScheme`] supporting markdown syntax.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,
    }
}

impl Http {
    /// Create new http authentication security schema.
    ///
    /// Accepts one argument which defines the scheme of the http authentication.
    ///
    /// # Examples
    ///
    /// Create http security schema with basic authentication.
    /// ```rust
    /// # use utoipa::openapi::security::{SecurityScheme, Http, HttpAuthScheme};
    /// SecurityScheme::Http(Http::new(HttpAuthScheme::Basic));
    /// ```
    pub fn new(scheme: HttpAuthScheme) -> Self {
        Self {
            scheme,
            bearer_format: None,
            description: None,
        }
    }
}

impl HttpBuilder {
    /// Add or change http authentication scheme used.
    ///
    /// # Examples
    ///
    /// Create new [`Http`] [`SecurityScheme`] via [`HttpBuilder`].
    /// ```rust
    /// # use utoipa::openapi::security::{HttpBuilder, HttpAuthScheme};
    /// let http = HttpBuilder::new().scheme(HttpAuthScheme::Basic).build();
    /// ```
    pub fn scheme(mut self, scheme: HttpAuthScheme) -> Self {
        self.scheme = scheme;

        self
    }
    /// Add or change informative bearer format for http security schema.
    ///
    /// This is only applicable to [`HttpAuthScheme::Bearer`].
    ///
    /// # Examples
    ///
    /// Add JTW bearer format for security schema.
    /// ```rust
    /// # use utoipa::openapi::security::{HttpBuilder, HttpAuthScheme};
    /// HttpBuilder::new().scheme(HttpAuthScheme::Bearer).bearer_format("JWT").build();
    /// ```
    pub fn bearer_format<S: Into<String>>(mut self, bearer_format: S) -> Self {
        if self.scheme == HttpAuthScheme::Bearer {
            self.bearer_format = Some(bearer_format.into());
        }

        self
    }

    /// Add or change optional description supporting markdown syntax.
    pub fn description<S: Into<String>>(mut self, description: Option<S>) -> Self {
        self.description = description.map(|description| description.into());

        self
    }
}

/// Implements types according [RFC7235](https://datatracker.ietf.org/doc/html/rfc7235#section-5.1).
///
/// Types are maintained at <https://www.iana.org/assignments/http-authschemes/http-authschemes.xhtml>.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "lowercase")]
pub enum HttpAuthScheme {
    Basic,
    Bearer,
    Digest,
    Hoba,
    Mutual,
    Negotiate,
    OAuth,
    #[serde(rename = "scram-sha-1")]
    ScramSha1,
    #[serde(rename = "scram-sha-256")]
    ScramSha256,
    Vapid,
}

impl Default for HttpAuthScheme {
    fn default() -> Self {
        Self::Basic
    }
}

/// Open id connect [`SecurityScheme`]
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct OpenIdConnect {
    /// Url of the [`OpenIdConnect`] to discover OAuth2 connect values.
    pub open_id_connect_url: String,

    /// Description of [`OpenIdConnect`] [`SecurityScheme`] supporting markdown syntax.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl OpenIdConnect {
    /// Construct a new open id connect security schema.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use utoipa::openapi::security::OpenIdConnect;
    /// OpenIdConnect::new("https://localhost/openid");
    /// ```
    pub fn new<S: Into<String>>(open_id_connect_url: S) -> Self {
        Self {
            open_id_connect_url: open_id_connect_url.into(),
            description: None,
        }
    }

    /// Construct a new [`OpenIdConnect`] [`SecurityScheme`] with optional description
    /// supporting markdown syntax.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use utoipa::openapi::security::OpenIdConnect;
    /// OpenIdConnect::with_description("https://localhost/openid", "my pet api open id connect");
    /// ```
    pub fn with_description<S: Into<String>>(open_id_connect_url: S, description: S) -> Self {
        Self {
            open_id_connect_url: open_id_connect_url.into(),
            description: Some(description.into()),
        }
    }
}

/// OAuth2 [`Flow`] configuration for [`SecurityScheme`].
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct OAuth2 {
    /// Map of supported OAuth2 flows.
    pub flows: BTreeMap<String, Flow>,

    /// Optional description for the [`OAuth2`] [`Flow`] [`SecurityScheme`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl OAuth2 {
    /// Construct a new OAuth2 security schema configuration object.
    ///
    /// Oauth flow accepts slice of [`Flow`] configuration objects and can be optionally provided with description.
    ///
    /// # Examples
    ///
    /// Create new OAuth2 flow with multiple authentication flows.
    /// ```rust
    /// # use utoipa::openapi::security::{OAuth2, Flow, Password, AuthorizationCode, Scopes};
    /// OAuth2::new([Flow::Password(
    ///     Password::with_refresh_url(
    ///         "https://localhost/oauth/token",
    ///         Scopes::from_iter([
    ///             ("edit:items", "edit my items"),
    ///             ("read:items", "read my items")
    ///         ]),
    ///         "https://localhost/refresh/token"
    ///     )),
    ///     Flow::AuthorizationCode(
    ///         AuthorizationCode::new(
    ///         "https://localhost/authorization/token",
    ///         "https://localhost/token/url",
    ///         Scopes::from_iter([
    ///             ("edit:items", "edit my items"),
    ///             ("read:items", "read my items")
    ///         ])),
    ///    ),
    /// ]);
    /// ```
    pub fn new<I: IntoIterator<Item = Flow>>(flows: I) -> Self {
        Self {
            flows: BTreeMap::from_iter(
                flows
                    .into_iter()
                    .map(|auth_flow| (String::from(auth_flow.get_type_as_str()), auth_flow)),
            ),
            description: None,
        }
    }

    /// Construct a new OAuth2 flow with optional description supporting markdown syntax.
    ///
    /// # Examples
    ///
    /// Create new OAuth2 flow with multiple authentication flows with description.
    /// ```rust
    /// # use utoipa::openapi::security::{OAuth2, Flow, Password, AuthorizationCode, Scopes};
    /// OAuth2::with_description([Flow::Password(
    ///     Password::with_refresh_url(
    ///         "https://localhost/oauth/token",
    ///         Scopes::from_iter([
    ///             ("edit:items", "edit my items"),
    ///             ("read:items", "read my items")
    ///         ]),
    ///         "https://localhost/refresh/token"
    ///     )),
    ///     Flow::AuthorizationCode(
    ///         AuthorizationCode::new(
    ///         "https://localhost/authorization/token",
    ///         "https://localhost/token/url",
    ///         Scopes::from_iter([
    ///             ("edit:items", "edit my items"),
    ///             ("read:items", "read my items")
    ///         ])
    ///      ),
    ///    ),
    /// ], "my oauth2 flow");
    /// ```
    pub fn with_description<I: IntoIterator<Item = Flow>, S: Into<String>>(
        flows: I,
        description: S,
    ) -> Self {
        Self {
            flows: BTreeMap::from_iter(
                flows
                    .into_iter()
                    .map(|auth_flow| (String::from(auth_flow.get_type_as_str()), auth_flow)),
            ),
            description: Some(description.into()),
        }
    }
}

/// [`OAuth2`] flow configuration object.
///
///
/// See more details at <https://spec.openapis.org/oas/latest.html#oauth-flows-object>.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(untagged)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum Flow {
    /// Define implicit [`Flow`] type. See [`Implicit::new`] for usage details.
    ///
    /// Soon to be deprecated by <https://datatracker.ietf.org/doc/html/draft-ietf-oauth-security-topics>.
    Implicit(Implicit),
    /// Define password [`Flow`] type. See [`Password::new`] for usage details.
    Password(Password),
    /// Define client credentials [`Flow`] type. See [`ClientCredentials::new`] for usage details.
    ClientCredentials(ClientCredentials),
    /// Define authorization code [`Flow`] type. See [`AuthorizationCode::new`] for usage details.
    AuthorizationCode(AuthorizationCode),
}

impl Flow {
    fn get_type_as_str(&self) -> &str {
        match self {
            Self::Implicit(_) => "implicit",
            Self::Password(_) => "password",
            Self::ClientCredentials(_) => "clientCredentials",
            Self::AuthorizationCode(_) => "authorizationCode",
        }
    }
}

/// Implicit [`Flow`] configuration for [`OAuth2`].
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Implicit {
    /// Authorization token url for the flow.
    pub authorization_url: String,

    /// Optional refresh token url for the flow.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<String>,

    /// Scopes required by the flow.
    #[serde(flatten)]
    pub scopes: Scopes,
}

impl Implicit {
    /// Construct a new implicit oauth2 flow.
    ///
    /// Accepts two arguments: one which is authorization url and second map of scopes. Scopes can
    /// also be an empty map.
    ///
    /// # Examples
    ///
    /// Create new implicit flow with scopes.
    /// ```rust
    /// # use utoipa::openapi::security::{Implicit, Scopes};
    /// Implicit::new(
    ///     "https://localhost/auth/dialog",
    ///     Scopes::from_iter([
    ///         ("edit:items", "edit my items"),
    ///         ("read:items", "read my items")
    ///     ]),
    /// );
    /// ```
    ///
    /// Create new implicit flow without any scopes.
    /// ```rust
    /// # use utoipa::openapi::security::{Implicit, Scopes};
    /// Implicit::new(
    ///     "https://localhost/auth/dialog",
    ///     Scopes::new(),
    /// );
    /// ```
    pub fn new<S: Into<String>>(authorization_url: S, scopes: Scopes) -> Self {
        Self {
            authorization_url: authorization_url.into(),
            refresh_url: None,
            scopes,
        }
    }

    /// Construct a new implicit oauth2 flow with refresh url for getting refresh tokens.
    ///
    /// This is essentially same as [`Implicit::new`] but allows defining `refresh_url` for the [`Implicit`]
    /// oauth2 flow.
    ///
    /// # Examples
    ///
    /// Create a new implicit oauth2 flow with refresh token.
    /// ```rust
    /// # use utoipa::openapi::security::{Implicit, Scopes};
    /// Implicit::with_refresh_url(
    ///     "https://localhost/auth/dialog",
    ///     Scopes::new(),
    ///     "https://localhost/refresh-token"
    /// );
    /// ```
    pub fn with_refresh_url<S: Into<String>>(
        authorization_url: S,
        scopes: Scopes,
        refresh_url: S,
    ) -> Self {
        Self {
            authorization_url: authorization_url.into(),
            refresh_url: Some(refresh_url.into()),
            scopes,
        }
    }
}

/// Authorization code [`Flow`] configuration for [`OAuth2`].
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct AuthorizationCode {
    /// Url for authorization token.
    pub authorization_url: String,
    /// Token url for the flow.
    pub token_url: String,

    /// Optional refresh token url for the flow.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<String>,

    /// Scopes required by the flow.
    #[serde(flatten)]
    pub scopes: Scopes,
}

impl AuthorizationCode {
    /// Construct a new authorization code oauth flow.
    ///
    /// Accepts three arguments: one which is authorization url, two a token url and
    /// three a map of scopes for oauth flow.
    ///
    /// # Examples
    ///
    /// Create new authorization code flow with scopes.
    /// ```rust
    /// # use utoipa::openapi::security::{AuthorizationCode, Scopes};
    /// AuthorizationCode::new(
    ///     "https://localhost/auth/dialog",
    ///     "https://localhost/token",
    ///     Scopes::from_iter([
    ///         ("edit:items", "edit my items"),
    ///         ("read:items", "read my items")
    ///     ]),
    /// );
    /// ```
    ///
    /// Create new authorization code flow without any scopes.
    /// ```rust
    /// # use utoipa::openapi::security::{AuthorizationCode, Scopes};
    /// AuthorizationCode::new(
    ///     "https://localhost/auth/dialog",
    ///     "https://localhost/token",
    ///     Scopes::new(),
    /// );
    /// ```
    pub fn new<A: Into<String>, T: Into<String>>(
        authorization_url: A,
        token_url: T,
        scopes: Scopes,
    ) -> Self {
        Self {
            authorization_url: authorization_url.into(),
            token_url: token_url.into(),
            refresh_url: None,
            scopes,
        }
    }

    /// Construct a new  [`AuthorizationCode`] OAuth2 flow with additional refresh token url.
    ///
    /// This is essentially same as [`AuthorizationCode::new`] but allows defining extra parameter `refresh_url`
    /// for fetching refresh token.
    ///
    /// # Examples
    ///
    /// Create [`AuthorizationCode`] OAuth2 flow with refresh url.
    /// ```rust
    /// # use utoipa::openapi::security::{AuthorizationCode, Scopes};
    /// AuthorizationCode::with_refresh_url(
    ///     "https://localhost/auth/dialog",
    ///     "https://localhost/token",
    ///     Scopes::new(),
    ///     "https://localhost/refresh-token"
    /// );
    /// ```
    pub fn with_refresh_url<S: Into<String>>(
        authorization_url: S,
        token_url: S,
        scopes: Scopes,
        refresh_url: S,
    ) -> Self {
        Self {
            authorization_url: authorization_url.into(),
            token_url: token_url.into(),
            refresh_url: Some(refresh_url.into()),
            scopes,
        }
    }
}

/// Password [`Flow`] configuration for [`OAuth2`].
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Password {
    /// Token url for this OAuth2 flow. OAuth2 standard requires TLS.
    pub token_url: String,

    /// Optional refresh token url.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<String>,

    /// Scopes required by the flow.
    #[serde(flatten)]
    pub scopes: Scopes,
}

impl Password {
    /// Construct a new password oauth flow.
    ///
    /// Accepts two arguments: one which is a token url and
    /// two a map of scopes for oauth flow.
    ///
    /// # Examples
    ///
    /// Create new password flow with scopes.
    /// ```rust
    /// # use utoipa::openapi::security::{Password, Scopes};
    /// Password::new(
    ///     "https://localhost/token",
    ///     Scopes::from_iter([
    ///         ("edit:items", "edit my items"),
    ///         ("read:items", "read my items")
    ///     ]),
    /// );
    /// ```
    ///
    /// Create new password flow without any scopes.
    /// ```rust
    /// # use utoipa::openapi::security::{Password, Scopes};
    /// Password::new(
    ///     "https://localhost/token",
    ///     Scopes::new(),
    /// );
    /// ```
    pub fn new<S: Into<String>>(token_url: S, scopes: Scopes) -> Self {
        Self {
            token_url: token_url.into(),
            refresh_url: None,
            scopes,
        }
    }

    /// Construct a new password oauth flow with additional refresh url.
    ///
    /// This is essentially same as [`Password::new`] but allows defining third parameter for `refresh_url`
    /// for fetching refresh tokens.
    ///
    /// # Examples
    ///
    /// Create new password flow with refresh url.
    /// ```rust
    /// # use utoipa::openapi::security::{Password, Scopes};
    /// Password::with_refresh_url(
    ///     "https://localhost/token",
    ///     Scopes::from_iter([
    ///         ("edit:items", "edit my items"),
    ///         ("read:items", "read my items")
    ///     ]),
    ///     "https://localhost/refres-token"
    /// );
    /// ```
    pub fn with_refresh_url<S: Into<String>>(token_url: S, scopes: Scopes, refresh_url: S) -> Self {
        Self {
            token_url: token_url.into(),
            refresh_url: Some(refresh_url.into()),
            scopes,
        }
    }
}

/// Client credentials [`Flow`] configuration for [`OAuth2`].
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ClientCredentials {
    /// Token url used for [`ClientCredentials`] flow. OAuth2 standard requires TLS.
    pub token_url: String,

    /// Optional refresh token url.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<String>,

    /// Scopes required by the flow.
    #[serde(flatten)]
    pub scopes: Scopes,
}

impl ClientCredentials {
    /// Construct a new client credentials oauth flow.
    ///
    /// Accepts two arguments: one which is a token url and
    /// two a map of scopes for oauth flow.
    ///
    /// # Examples
    ///
    /// Create new client credentials flow with scopes.
    /// ```rust
    /// # use utoipa::openapi::security::{ClientCredentials, Scopes};
    /// ClientCredentials::new(
    ///     "https://localhost/token",
    ///     Scopes::from_iter([
    ///         ("edit:items", "edit my items"),
    ///         ("read:items", "read my items")
    ///     ]),
    /// );
    /// ```
    ///
    /// Create new client credentials flow without any scopes.
    /// ```rust
    /// # use utoipa::openapi::security::{ClientCredentials, Scopes};
    /// ClientCredentials::new(
    ///     "https://localhost/token",
    ///     Scopes::new(),
    /// );
    /// ```
    pub fn new<S: Into<String>>(token_url: S, scopes: Scopes) -> Self {
        Self {
            token_url: token_url.into(),
            refresh_url: None,
            scopes,
        }
    }

    /// Construct a new client credentials oauth flow with additional refresh url.
    ///
    /// This is essentially same as [`ClientCredentials::new`] but allows defining third parameter for
    /// `refresh_url`.
    ///
    /// # Examples
    ///
    /// Create new client credentials for with refresh url.
    /// ```rust
    /// # use utoipa::openapi::security::{ClientCredentials, Scopes};
    /// ClientCredentials::with_refresh_url(
    ///     "https://localhost/token",
    ///     Scopes::from_iter([
    ///         ("edit:items", "edit my items"),
    ///         ("read:items", "read my items")
    ///     ]),
    ///     "https://localhost/refresh-url"
    /// );
    /// ```
    pub fn with_refresh_url<S: Into<String>>(token_url: S, scopes: Scopes, refresh_url: S) -> Self {
        Self {
            token_url: token_url.into(),
            refresh_url: Some(refresh_url.into()),
            scopes,
        }
    }
}

/// [`OAuth2`] flow scopes object defines required permissions for oauth flow.
///
/// Scopes must be given to oauth2 flow but depending on need one of few initialization methods
/// could be used.
///
/// * Create empty map of scopes you can use [`Scopes::new`].
/// * Create map with only one scope you can use [`Scopes::one`].
/// * Create multiple scopes from iterator with [`Scopes::from_iter`].
///
/// # Examples
///
/// Create empty map of scopes.
/// ```rust
/// # use utoipa::openapi::security::Scopes;
/// let scopes = Scopes::new();
/// ```
///
/// Create [`Scopes`] holding one scope.
/// ```rust
/// # use utoipa::openapi::security::Scopes;
/// let scopes = Scopes::one("edit:item", "edit pets");
/// ```
///
/// Create map of scopes from iterator.
/// ```rust
/// # use utoipa::openapi::security::Scopes;
/// let scopes = Scopes::from_iter([
///     ("edit:items", "edit my items"),
///     ("read:items", "read my items")
/// ]);
/// ```
#[derive(Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Scopes {
    scopes: BTreeMap<String, String>,
}

impl Scopes {
    /// Construct new [`Scopes`] with empty map of scopes. This is useful if oauth flow does not need
    /// any permission scopes.
    ///
    /// # Examples
    ///
    /// Create empty map of scopes.
    /// ```rust
    /// # use utoipa::openapi::security::Scopes;
    /// let scopes = Scopes::new();
    /// ```
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Construct new [`Scopes`] with holding one scope.
    ///
    /// * `scope` Is be the permission required.
    /// * `description` Short description about the permission.
    ///
    /// # Examples
    ///
    /// Create map of scopes with one scope item.
    /// ```rust
    /// # use utoipa::openapi::security::Scopes;
    /// let scopes = Scopes::one("edit:item", "edit items");
    /// ```
    pub fn one<S: Into<String>>(scope: S, description: S) -> Self {
        Self {
            scopes: BTreeMap::from_iter(iter::once_with(|| (scope.into(), description.into()))),
        }
    }
}

impl<I> FromIterator<(I, I)> for Scopes
where
    I: Into<String>,
{
    fn from_iter<T: IntoIterator<Item = (I, I)>>(iter: T) -> Self {
        Self {
            scopes: iter
                .into_iter()
                .map(|(key, value)| (key.into(), value.into()))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_fn {
        ($name:ident: $schema:expr; $expected:literal) => {
            #[test]
            fn $name() {
                let value = serde_json::to_value($schema).unwrap();
                let expected_value: serde_json::Value = serde_json::from_str($expected).unwrap();

                assert_eq!(
                    value,
                    expected_value,
                    "testing serializing \"{}\": \nactual:\n{}\nexpected:\n{}",
                    stringify!($name),
                    value,
                    expected_value
                );

                println!("{}", &serde_json::to_string_pretty(&$schema).unwrap());
            }
        };
    }

    test_fn! {
    security_schema_correct_http_bearer_json:
    SecurityScheme::Http(
        HttpBuilder::new().scheme(HttpAuthScheme::Bearer).bearer_format("JWT").build()
    );
    r###"{
  "type": "http",
  "scheme": "bearer",
  "bearerFormat": "JWT"
}"###
    }

    test_fn! {
        security_schema_correct_basic_auth:
        SecurityScheme::Http(Http::new(HttpAuthScheme::Basic));
        r###"{
  "type": "http",
  "scheme": "basic"
}"###
    }

    test_fn! {
        security_schema_correct_digest_auth:
        SecurityScheme::Http(Http::new(HttpAuthScheme::Digest));
        r###"{
  "type": "http",
  "scheme": "digest"
}"###
    }

    test_fn! {
        security_schema_correct_hoba_auth:
        SecurityScheme::Http(Http::new(HttpAuthScheme::Hoba));
        r###"{
  "type": "http",
  "scheme": "hoba"
}"###
    }

    test_fn! {
        security_schema_correct_mutual_auth:
        SecurityScheme::Http(Http::new(HttpAuthScheme::Mutual));
        r###"{
  "type": "http",
  "scheme": "mutual"
}"###
    }

    test_fn! {
        security_schema_correct_negotiate_auth:
        SecurityScheme::Http(Http::new(HttpAuthScheme::Negotiate));
        r###"{
  "type": "http",
  "scheme": "negotiate"
}"###
    }

    test_fn! {
        security_schema_correct_oauth_auth:
        SecurityScheme::Http(Http::new(HttpAuthScheme::OAuth));
        r###"{
  "type": "http",
  "scheme": "oauth"
}"###
    }

    test_fn! {
        security_schema_correct_scram_sha1_auth:
        SecurityScheme::Http(Http::new(HttpAuthScheme::ScramSha1));
        r###"{
  "type": "http",
  "scheme": "scram-sha-1"
}"###
    }

    test_fn! {
        security_schema_correct_scram_sha256_auth:
        SecurityScheme::Http(Http::new(HttpAuthScheme::ScramSha256));
        r###"{
  "type": "http",
  "scheme": "scram-sha-256"
}"###
    }

    test_fn! {
        security_schema_correct_api_key_cookie_auth:
        SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new(String::from("api_key"))));
        r###"{
  "type": "apiKey",
  "name": "api_key",
  "in": "cookie"
}"###
    }

    test_fn! {
        security_schema_correct_api_key_header_auth:
        SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("api_key")));
        r###"{
  "type": "apiKey",
  "name": "api_key",
  "in": "header"
}"###
    }

    test_fn! {
        security_schema_correct_api_key_query_auth:
        SecurityScheme::ApiKey(ApiKey::Query(ApiKeyValue::new(String::from("api_key"))));
        r###"{
  "type": "apiKey",
  "name": "api_key",
  "in": "query"
}"###
    }

    test_fn! {
        security_schema_correct_open_id_connect_auth:
        SecurityScheme::OpenIdConnect(OpenIdConnect::new("https://localhost/openid"));
        r###"{
  "type": "openIdConnect",
  "openIdConnectUrl": "https://localhost/openid"
}"###
    }

    test_fn! {
        security_schema_correct_oauth2_implicit:
        SecurityScheme::OAuth2(
            OAuth2::with_description([Flow::Implicit(
                Implicit::new(
                    "https://localhost/auth/dialog",
                    Scopes::from_iter([
                        ("edit:items", "edit my items"),
                        ("read:items", "read my items")
                    ]),
                ),
            )], "my oauth2 flow")
        );
        r###"{
  "type": "oauth2",
  "flows": {
    "implicit": {
      "authorizationUrl": "https://localhost/auth/dialog",
      "scopes": {
        "edit:items": "edit my items",
        "read:items": "read my items"
      }
    }
  },
  "description": "my oauth2 flow"
}"###
    }

    test_fn! {
        security_schema_correct_oauth2_password:
        SecurityScheme::OAuth2(
            OAuth2::with_description([Flow::Password(
                Password::with_refresh_url(
                    "https://localhost/oauth/token",
                    Scopes::from_iter([
                        ("edit:items", "edit my items"),
                        ("read:items", "read my items")
                    ]),
                    "https://localhost/refresh/token"
                ),
            )], "my oauth2 flow")
        );
        r###"{
  "type": "oauth2",
  "flows": {
    "password": {
      "tokenUrl": "https://localhost/oauth/token",
      "refreshUrl": "https://localhost/refresh/token",
      "scopes": {
        "edit:items": "edit my items",
        "read:items": "read my items"
      }
    }
  },
  "description": "my oauth2 flow"
}"###
    }

    test_fn! {
        security_schema_correct_oauth2_client_credentials:
        SecurityScheme::OAuth2(
            OAuth2::new([Flow::ClientCredentials(
                ClientCredentials::with_refresh_url(
                    "https://localhost/oauth/token",
                    Scopes::from_iter([
                        ("edit:items", "edit my items"),
                        ("read:items", "read my items")
                    ]),
                    "https://localhost/refresh/token"
                ),
            )])
        );
        r###"{
  "type": "oauth2",
  "flows": {
    "clientCredentials": {
      "tokenUrl": "https://localhost/oauth/token",
      "refreshUrl": "https://localhost/refresh/token",
      "scopes": {
        "edit:items": "edit my items",
        "read:items": "read my items"
      }
    }
  }
}"###
    }

    test_fn! {
        security_schema_correct_oauth2_authorization_code:
        SecurityScheme::OAuth2(
            OAuth2::new([Flow::AuthorizationCode(
                AuthorizationCode::with_refresh_url(
                    "https://localhost/authorization/token",
                    "https://localhost/token/url",
                    Scopes::from_iter([
                        ("edit:items", "edit my items"),
                        ("read:items", "read my items")
                    ]),
                    "https://localhost/refresh/token"
                ),
            )])
        );
        r###"{
  "type": "oauth2",
  "flows": {
    "authorizationCode": {
      "authorizationUrl": "https://localhost/authorization/token",
      "tokenUrl": "https://localhost/token/url",
      "refreshUrl": "https://localhost/refresh/token",
      "scopes": {
        "edit:items": "edit my items",
        "read:items": "read my items"
      }
    }
  }
}"###
    }

    test_fn! {
        security_schema_correct_oauth2_authorization_code_no_scopes:
        SecurityScheme::OAuth2(
            OAuth2::new([Flow::AuthorizationCode(
                AuthorizationCode::with_refresh_url(
                    "https://localhost/authorization/token",
                    "https://localhost/token/url",
                    Scopes::new(),
                    "https://localhost/refresh/token"
                ),
            )])
        );
        r###"{
  "type": "oauth2",
  "flows": {
    "authorizationCode": {
      "authorizationUrl": "https://localhost/authorization/token",
      "tokenUrl": "https://localhost/token/url",
      "refreshUrl": "https://localhost/refresh/token",
      "scopes": {}
    }
  }
}"###
    }

    test_fn! {
        security_schema_correct_mutual_tls:
        SecurityScheme::MutualTls {
            description: Some(String::from("authorization is performed with client side certificate"))
        };
        r###"{
  "type": "mutualTLS",
  "description": "authorization is performed with client side certificate"
}"###
    }
}
