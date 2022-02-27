//! OpenAPI schema's security components implementations.
//!
//! Refer to [`SecuritySchema`] for usage and more details.
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SecurityRequirement {
    #[serde(flatten)]
    value: HashMap<String, Vec<String>>,
}

impl SecurityRequirement {
    pub fn new() -> Self {
        Default::default()
    }
}

/// Defines OpenAPI security schema that the path operations can use.
///
/// See more details at <https://spec.openapis.org/oas/latest.html#security-scheme-object>.
///
/// # Examples
///
/// Create implicit oauth2 flow security schema for path operations.
/// ```rust
/// # use utoipa::openapi::security::{SecuritySchema, Oauth2, Implicit, Flow};
/// # use std::collections::HashMap;
/// SecuritySchema::Oauth2(
///     Oauth2::new([Flow::Implicit(
///         Implicit::new(
///             "http://localhost/auth/dialog",
///             HashMap::from([
///                 ("edit:items".to_string(), "edit my items".to_string()),
///                 ("read:items".to_string(), "read my items".to_string()
///             )]),
///         ),
///     )]).with_description("my oauth2 flow")
/// );
/// ```
///
/// Create JWT header authetication.
/// ```rust
/// # use utoipa::openapi::security::{SecuritySchema, HttpAuthenticationType, Http};
/// SecuritySchema::Http(
///     Http::new(HttpAuthenticationType::Bearer).with_bearer_format("JWT")
/// );
/// ```
#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum SecuritySchema {
    /// Oauth flow authentication.
    Oauth2(Oauth2),
    /// Api key authentication sent in *`header`*, *`cookie`* or *`query`*.
    ApiKey(ApiKey),
    /// Http authentication such as *`bearer`* or *`basic`*.
    Http(Http),
    /// Open id connect url to discover OAuth2 configuraiton values.
    OpenIdConnect(OpenIdConnect),
    /// Authentication is done via client side cerfiticate.
    ///
    /// OpenApi 3.1 type
    #[serde(rename = "mutualTLS")]
    MutualTls {
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
}

/// Api key authentication [`SecuritySchema`].
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ApiKey {
    name: String,

    #[serde(rename = "in")]
    api_key_in: ApiKeyIn,

    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

impl ApiKey {
    /// Constructs new api key authentication schema.
    ///
    /// Accepts two arguments: one which is the name of parameter and second which defines where
    /// the parameter is defined in.
    ///
    /// # Examples
    ///
    /// Create new api key security schema with parameter name `api_key` which must be present with value in
    /// header.
    /// ```rust
    /// # use utoipa::openapi::security::{ApiKey, ApiKeyIn};
    /// let api_key = ApiKey::new("api_key", ApiKeyIn::Header);
    /// ```
    pub fn new<S: Into<String>>(name: S, api_key_in: ApiKeyIn) -> Self {
        Self {
            name: name.into(),
            api_key_in,
            description: None,
        }
    }

    /// Optional description supporting markdown syntax.
    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());

        self
    }
}

/// Define the location where api key must be provided.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum ApiKeyIn {
    Header,
    Query,
    Cookie,
}

/// Http authentication [`SecuritySchema`].
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Http {
    scheme: HttpAuthenticationType,

    #[serde(skip_serializing_if = "Option::is_none")]
    bearer_format: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

impl Http {
    /// Create new http authentication security schema.
    ///
    /// Accepts one argument which defines the scheme of the http authentication.
    ///
    /// # Examples
    ///
    /// Create http securith schema with basic authentication.
    /// ```rust
    /// # use utoipa::openapi::security::{SecuritySchema, Http, HttpAuthenticationType};
    /// SecuritySchema::Http(Http::new(HttpAuthenticationType::Basic));
    /// ```
    pub fn new(scheme: HttpAuthenticationType) -> Self {
        Self {
            scheme,
            bearer_format: None,
            description: None,
        }
    }

    /// Add informative bearer format for http security schema.
    ///
    /// This is no-op in any other [`HttpAuthenticationType`] than [`HttpAuthenticationType::Bearer`].
    ///
    /// # Examples
    ///
    /// Add JTW bearer format for security schema.
    /// ```rust
    /// # use utoipa::openapi::security::{Http, HttpAuthenticationType};
    /// Http::new(HttpAuthenticationType::Bearer).with_bearer_format("JWT");
    /// ```
    pub fn with_bearer_format<S: Into<String>>(mut self, bearer_format: S) -> Self {
        if self.scheme == HttpAuthenticationType::Bearer {
            self.bearer_format = Some(bearer_format.into());
        }

        self
    }

    /// Optional description supporting markdown syntax.
    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());

        self
    }
}

/// Implements types according [RFC7235](https://datatracker.ietf.org/doc/html/rfc7235#section-5.1).
///
/// Types are maintainted at <https://www.iana.org/assignments/http-authschemes/http-authschemes.xhtml>.
#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "lowercase")]
pub enum HttpAuthenticationType {
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

/// Open id connect [`SecuritySchema`]
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct OpenIdConnect {
    open_id_connect_url: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

impl OpenIdConnect {
    /// Construct a new open id connect security schema.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use utoipa::openapi::security::OpenIdConnect;
    /// OpenIdConnect::new("http://localhost/openid");
    /// ```
    pub fn new<S: Into<String>>(open_id_connect_url: S) -> Self {
        Self {
            open_id_connect_url: open_id_connect_url.into(),
            description: None,
        }
    }

    /// Optional description supporting markdown syntax.
    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());

        self
    }
}

/// OAuth2 [`Flow`] configuration for [`SecuritySchema`].
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Oauth2 {
    flows: HashMap<String, Flow>,

    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

impl Oauth2 {
    /// Construct a new Oauth2 security schema configuration object.
    ///
    /// Oauth flow accepts slice of [`Flow`] configuration objects and can be optionally provided with description.
    ///
    /// # Examples
    ///
    /// Create new OAuth2 flow with multiple authentication flows.
    /// ```rust
    /// # use std::collections::HashMap;
    /// # use utoipa::openapi::security::{Oauth2, Flow, Password, AuthorizationCode};
    /// Oauth2::new([Flow::Password(
    ///     Password::new(
    ///         "http://localhost/oauth/token",
    ///         HashMap::from([
    ///             ("edit:items".to_string(), "edit my items".to_string()),
    ///             ("read:items".to_string(), "read my items".to_string()
    ///         )]),
    ///     ).with_refresh_url("http://localhost/refresh/token")),
    ///     Flow::AuthorizationCode(
    ///         AuthorizationCode::new(
    ///         "http://localhost/authorization/token",
    ///         "http://localhost/token/url",
    ///         HashMap::from([
    ///             ("edit:items".to_string(), "edit my items".to_string()),
    ///             ("read:items".to_string(), "read my items".to_string()
    ///         )])),
    ///    ),
    /// ]).with_description("my oauth2 flow");
    /// ```
    pub fn new<I: IntoIterator<Item = Flow>>(flows: I) -> Self {
        Self {
            flows: HashMap::from_iter(
                flows
                    .into_iter()
                    .map(|auth_flow| (String::from(auth_flow.get_type_as_str()), auth_flow)),
            ),
            description: None,
        }
    }

    /// Optional description supporting markdown syntax.
    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());

        self
    }
}

/// [`Oauth2`] flow configuration object.
///
///
/// See more details at <https://spec.openapis.org/oas/latest.html#oauth-flows-object>.
#[derive(Serialize, Deserialize, Clone)]
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

/// Implicit [`Flow`] configuration for [`Oauth2`].
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Implicit {
    authorization_url: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_url: Option<String>,

    scopes: HashMap<String, String>,
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
    /// # use std::collections::HashMap;
    /// # use utoipa::openapi::security::Implicit;
    /// Implicit::new(
    ///     "http://localhost/auth/dialog",
    ///     HashMap::from([
    ///         ("edit:items".to_string(), "edit my items".to_string()),
    ///         ("read:items".to_string(), "read my items".to_string()
    ///     )]),
    /// );
    /// ```
    ///
    /// Create new implicit flow without any scopes.
    /// ```rust
    /// # use std::collections::HashMap;
    /// # use utoipa::openapi::security::Implicit;
    /// Implicit::new(
    ///     "http://localhost/auth/dialog",
    ///     HashMap::new(),
    /// );
    /// ```
    pub fn new<S: Into<String>>(authorization_url: S, scopes: HashMap<String, String>) -> Self {
        Self {
            authorization_url: authorization_url.into(),
            refresh_url: None,
            scopes,
        }
    }

    /// Add refresh url for getting refresh tokens.
    pub fn with_refresh_url<S: Into<String>>(mut self, refresh_url: S) -> Self {
        self.refresh_url = Some(refresh_url.into());

        self
    }
}

/// Authorization code [`Flow`] configuration for [`Oauth2`].
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct AuthorizationCode {
    authorization_url: String,
    token_url: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_url: Option<String>,

    scopes: HashMap<String, String>,
}

impl AuthorizationCode {
    /// Construct a new authorization code oauth flow.
    ///
    /// Accpets three arguments: one which is authorization url, two a token url and
    /// three a map of scopes for oauth flow.
    ///
    /// # Examples
    ///
    /// Create new authorization code flow with scopes.
    /// ```rust
    /// # use std::collections::HashMap;
    /// # use utoipa::openapi::security::AuthorizationCode;
    /// AuthorizationCode::new(
    ///     "http://localhost/auth/dialog",
    ///     "http://localhost/token",
    ///     HashMap::from([
    ///         ("edit:items".to_string(), "edit my items".to_string()),
    ///         ("read:items".to_string(), "read my items".to_string()
    ///     )]),
    /// );
    /// ```
    ///
    /// Create new authorization code flow without any scopes.
    /// ```rust
    /// # use std::collections::HashMap;
    /// # use utoipa::openapi::security::AuthorizationCode;
    /// AuthorizationCode::new(
    ///     "http://localhost/auth/dialog",
    ///     "http://localhost/token",
    ///     HashMap::new(),
    /// );
    /// ```
    pub fn new<A: Into<String>, T: Into<String>>(
        authorization_url: A,
        token_url: T,
        scopes: HashMap<String, String>,
    ) -> Self {
        Self {
            authorization_url: authorization_url.into(),
            token_url: token_url.into(),
            refresh_url: None,
            scopes,
        }
    }

    /// Add refresh url for getting refresh tokens.
    pub fn with_refresh_url<S: Into<String>>(mut self, refresh_url: S) -> Self {
        self.refresh_url = Some(refresh_url.into());

        self
    }
}

/// Password [`Flow`] configuration for [`Oauth2`].
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Password {
    token_url: String,
    refresh_url: Option<String>,
    scopes: HashMap<String, String>,
}

impl Password {
    /// Construct a new password oauth flow.
    ///
    /// Accpets two arguments: one which is a token url and
    /// two a map of scopes for oauth flow.
    ///
    /// # Examples
    ///
    /// Create new password flow with scopes.
    /// ```rust
    /// # use std::collections::HashMap;
    /// # use utoipa::openapi::security::Password;
    /// Password::new(
    ///     "http://localhost/token",
    ///     HashMap::from([
    ///         ("edit:items".to_string(), "edit my items".to_string()),
    ///         ("read:items".to_string(), "read my items".to_string()
    ///     )]),
    /// );
    /// ```
    ///
    /// Create new password flow without any scopes.
    /// ```rust
    /// # use std::collections::HashMap;
    /// # use utoipa::openapi::security::Password;
    /// Password::new(
    ///     "http://localhost/token",
    ///     HashMap::new(),
    /// );
    /// ```
    pub fn new<S: Into<String>>(token_url: S, scopes: HashMap<String, String>) -> Self {
        Self {
            token_url: token_url.into(),
            refresh_url: None,
            scopes,
        }
    }

    /// Add refresh url for getting refresh tokens.
    pub fn with_refresh_url<S: Into<String>>(mut self, refresh_url: S) -> Self {
        self.refresh_url = Some(refresh_url.into());

        self
    }
}

/// Client credentials [`Flow`] configuration for [`Oauth2`].
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ClientCredentials {
    token_url: String,
    refresh_url: Option<String>,
    scopes: HashMap<String, String>,
}

impl ClientCredentials {
    /// Construct a new client crendentials oauth flow.
    ///
    /// Accpets two arguments: one which is a token url and
    /// two a map of scopes for oauth flow.
    ///
    /// # Examples
    ///
    /// Create new client credentials flow with scopes.
    /// ```rust
    /// # use std::collections::HashMap;
    /// # use utoipa::openapi::security::ClientCredentials;
    /// ClientCredentials::new(
    ///     "http://localhost/token",
    ///     HashMap::from([
    ///         ("edit:items".to_string(), "edit my items".to_string()),
    ///         ("read:items".to_string(), "read my items".to_string()
    ///     )]),
    /// );
    /// ```
    ///
    /// Create new client credentials flow without any scopes.
    /// ```rust
    /// # use std::collections::HashMap;
    /// # use utoipa::openapi::security::ClientCredentials;
    /// ClientCredentials::new(
    ///     "http://localhost/token",
    ///     HashMap::new(),
    /// );
    /// ```
    pub fn new<S: Into<String>>(token_url: S, scopes: HashMap<String, String>) -> Self {
        Self {
            token_url: token_url.into(),
            refresh_url: None,
            scopes,
        }
    }

    /// Add refresh url for getting refresh tokens.
    pub fn with_refresh_url<S: Into<String>>(mut self, refresh_url: S) -> Self {
        self.refresh_url = Some(refresh_url.into());

        self
    }
}

#[cfg(test)]
#[cfg(feature = "json")]
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
    SecuritySchema::Http(
        Http::new(HttpAuthenticationType::Bearer).with_bearer_format("JWT")
    );
    r###"{
  "type": "http",
  "scheme": "bearer",
  "bearerFormat": "JWT"
}"###
    }

    test_fn! {
        security_schema_correct_basic_auth:
        SecuritySchema::Http(Http::new(HttpAuthenticationType::Basic));
        r###"{
  "type": "http",
  "scheme": "basic"
}"###
    }

    test_fn! {
        security_schema_correct_digest_auth:
        SecuritySchema::Http(Http::new(HttpAuthenticationType::Digest));
        r###"{
  "type": "http",
  "scheme": "digest"
}"###
    }

    test_fn! {
        security_schema_correct_hoba_auth:
        SecuritySchema::Http(Http::new(HttpAuthenticationType::Hoba));
        r###"{
  "type": "http",
  "scheme": "hoba"
}"###
    }

    test_fn! {
        security_schema_correct_mutual_auth:
        SecuritySchema::Http(Http::new(HttpAuthenticationType::Mutual));
        r###"{
  "type": "http",
  "scheme": "mutual"
}"###
    }

    test_fn! {
        security_schema_correct_negotiate_auth:
        SecuritySchema::Http(Http::new(HttpAuthenticationType::Negotiate));
        r###"{
  "type": "http",
  "scheme": "negotiate"
}"###
    }

    test_fn! {
        security_schema_correct_oauth_auth:
        SecuritySchema::Http(Http::new(HttpAuthenticationType::OAuth));
        r###"{
  "type": "http",
  "scheme": "oauth"
}"###
    }

    test_fn! {
        security_schema_correct_scram_sha1_auth:
        SecuritySchema::Http(Http::new(HttpAuthenticationType::ScramSha1));
        r###"{
  "type": "http",
  "scheme": "scram-sha-1"
}"###
    }

    test_fn! {
        security_schema_correct_scram_sha256_auth:
        SecuritySchema::Http(Http::new(HttpAuthenticationType::ScramSha256));
        r###"{
  "type": "http",
  "scheme": "scram-sha-256"
}"###
    }

    test_fn! {
        security_schema_correct_api_key_cookie_auth:
        SecuritySchema::ApiKey(ApiKey::new(String::from("api_key"), ApiKeyIn::Cookie));
        r###"{
  "type": "apiKey",
  "name": "api_key",
  "in": "cookie"
}"###
    }

    test_fn! {
        security_schema_correct_api_key_header_auth:
        SecuritySchema::ApiKey(ApiKey::new("api_key", ApiKeyIn::Header));
        r###"{
  "type": "apiKey",
  "name": "api_key",
  "in": "header"
}"###
    }

    test_fn! {
        security_schema_correct_api_key_query_auth:
        SecuritySchema::ApiKey(ApiKey::new(String::from("api_key"), ApiKeyIn::Query));
        r###"{
  "type": "apiKey",
  "name": "api_key",
  "in": "query"
}"###
    }

    test_fn! {
        security_schema_correct_open_id_connect_auth:
        SecuritySchema::OpenIdConnect(OpenIdConnect::new("http://localhost/openid"));
        r###"{
  "type": "openIdConnect",
  "openIdConnectUrl": "http://localhost/openid"
}"###
    }

    test_fn! {
        security_schema_correct_oauth2_implicit:
        SecuritySchema::Oauth2(
            Oauth2::new([Flow::Implicit(
                Implicit::new(
                    "http://localhost/auth/dialog",
                    HashMap::from([
                        ("edit:items".to_string(), "edit my items".to_string()),
                        ("read:items".to_string(), "read my items".to_string()
                    )]),
                ),
            )]).with_description("my oauth2 flow")
        );
        r###"{
  "type": "oauth2",
  "flows": {
    "implicit": {
      "authorizationUrl": "http://localhost/auth/dialog",
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
        SecuritySchema::Oauth2(
            Oauth2::new([Flow::Password(
                Password::new(
                    "http://localhost/oauth/token",
                    HashMap::from([
                        ("edit:items".to_string(), "edit my items".to_string()),
                        ("read:items".to_string(), "read my items".to_string()
                    )]),
                ).with_refresh_url("http://localhost/refresh/token"),
            )]).with_description("my oauth2 flow")
        );
        r###"{
  "type": "oauth2",
  "flows": {
    "password": {
      "tokenUrl": "http://localhost/oauth/token",
      "refreshUrl": "http://localhost/refresh/token",
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
        SecuritySchema::Oauth2(
            Oauth2::new([Flow::ClientCredentials(
                ClientCredentials::new(
                    "http://localhost/oauth/token",
                    HashMap::from([
                        ("edit:items".to_string(), "edit my items".to_string()),
                        ("read:items".to_string(), "read my items".to_string()
                    )]),
                ).with_refresh_url("http://localhost/refresh/token"),
            )]).with_description("my oauth2 flow")
        );
        r###"{
  "type": "oauth2",
  "flows": {
    "clientCredentials": {
      "tokenUrl": "http://localhost/oauth/token",
      "refreshUrl": "http://localhost/refresh/token",
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
        security_schema_correct_oauth2_authorization_code:
        SecuritySchema::Oauth2(
            Oauth2::new([Flow::AuthorizationCode(
                AuthorizationCode::new(
                    "http://localhost/authorization/token",
                    "http://localhost/token/url",
                    HashMap::from([
                        ("edit:items".to_string(), "edit my items".to_string()),
                        ("read:items".to_string(), "read my items".to_string()
                    )]),
                ).with_refresh_url("http://localhost/refresh/token"),
            )]).with_description("my oauth2 flow")
        );
        r###"{
  "type": "oauth2",
  "flows": {
    "authorizationCode": {
      "authorizationUrl": "http://localhost/authorization/token",
      "tokenUrl": "http://localhost/token/url",
      "refreshUrl": "http://localhost/refresh/token",
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
        security_schema_correct_oauth2_authorization_code_no_scopes:
        SecuritySchema::Oauth2(
            Oauth2::new([Flow::AuthorizationCode(
                AuthorizationCode::new(
                    "http://localhost/authorization/token",
                    "http://localhost/token/url",
                    HashMap::new(),
                ).with_refresh_url("http://localhost/refresh/token"),
            )])
        );
        r###"{
  "type": "oauth2",
  "flows": {
    "authorizationCode": {
      "authorizationUrl": "http://localhost/authorization/token",
      "tokenUrl": "http://localhost/token/url",
      "refreshUrl": "http://localhost/refresh/token",
      "scopes": {}
    }
  }
}"###
    }

    test_fn! {
        security_schema_correct_mutual_tls:
        SecuritySchema::MutualTls {
            description: Some(String::from("authorizaion is performed with client side certificate"))
        };
        r###"{
  "type": "mutualTLS",
  "description": "authorizaion is performed with client side certificate"
}"###
    }
}
