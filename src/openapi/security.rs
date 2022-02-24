use std::{collections::HashMap, hash::Hash};

use actix_web::client::Client;
use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Security {
    #[serde(flatten)]
    pub value: HashMap<String, Vec<String>>,
}

impl Security {
    pub fn new() -> Self {
        Default::default()
    }
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct SecuritySchema {
    #[serde(rename = "type")]
    schema_type: String,

    #[serde(skip_serializing_if = "String::is_empty")]
    description: String,

    #[serde(flatten)]
    security_type: SecurityType,
}

impl SecuritySchema {
    pub fn new(security_type: SecurityType) -> Self {
        Self {
            schema_type: String::from(SecuritySchema::resolve_type(&security_type)),
            description: String::new(),
            security_type,
        }
    }

    fn resolve_type(security_type: &SecurityType) -> &str {
        match security_type {
            SecurityType::OAuth2 { .. } => "oauth2",
            SecurityType::ApiKey { .. } => "apiKey",
            SecurityType::Http { .. } => "http",
            SecurityType::OpenIdConnect { .. } => "openIdConnect",
        }
    }

    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = description.into();

        self
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum SecurityType {
    #[serde(rename_all = "camelCase")]
    OAuth2 {
        #[serde(flatten)]
        flows: Flows,
    },
    #[serde(rename_all = "camelCase")]
    ApiKey {
        name: String,
        #[serde(rename = "in")]
        api_key_in: ApiKeyIn,
    },
    #[serde(rename_all = "camelCase")]
    Http {
        schema: HttpAutheticationType,
        #[serde(skip_serializing_if = "Option::is_none")]
        bearer_format: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    OpenIdConnect { open_id_connect_url: String },
}

/// Implements types according [RFC7235](https://datatracker.ietf.org/doc/html/rfc7235#section-5.1) which are maintained in
/// https://www.iana.org/assignments/http-authschemes/http-authschemes.xhtml
#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "lowercase")]
pub enum HttpAutheticationType {
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

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum ApiKeyIn {
    Header,
    Query,
    Cookie,
}

#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Flows {
    flows: HashMap<String, Flow>,
}

impl Flows {
    pub fn new<I: IntoIterator<Item = Flow>>(flows: I) -> Self {
        Self {
            flows: HashMap::from_iter(
                flows
                    .into_iter()
                    .map(|auth| (String::from(auth.get_type_as_str()), auth)),
            ),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum Flow {
    Implicit(Implicit),
    Password(Password),
    ClientCredentials(ClientCredentials),
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
    pub fn new<S: Into<String>>(authorization_url: S, scopes: HashMap<String, String>) -> Self {
        Self {
            authorization_url: authorization_url.into(),
            refresh_url: None,
            scopes,
        }
    }

    pub fn with_refresh_url<S: Into<String>>(mut self, refresh_url: S) -> Self {
        self.refresh_url = Some(refresh_url.into());

        self
    }
}

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
    pub fn new<A: Into<String>, T: Into<String>, R: Into<String>>(
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

    pub fn with_refresh_url<S: Into<String>>(mut self, refresh_url: S) -> Self {
        self.refresh_url = Some(refresh_url.into());

        self
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Password {
    token_url: String,
    refresh_url: Option<String>,
    scopes: HashMap<String, String>,
}

impl Password {
    pub fn new<S: Into<String>>(token_url: S, scopes: HashMap<String, String>) -> Self {
        Self {
            token_url: token_url.into(),
            refresh_url: None,
            scopes,
        }
    }

    pub fn with_refresh_url<S: Into<String>>(mut self, refresh_url: S) -> Self {
        self.refresh_url = Some(refresh_url.into());

        self
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ClientCredentials {
    token_url: String,
    refresh_url: Option<String>,
    scopes: HashMap<String, String>,
}

impl ClientCredentials {
    pub fn new<S: Into<String>>(token_url: S, scopes: HashMap<String, String>) -> Self {
        Self {
            token_url: token_url.into(),
            refresh_url: None,
            scopes,
        }
    }

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
    SecuritySchema::new(SecurityType::Http {
                bearer_format: Some("JWT".to_string()),
                schema: HttpAutheticationType::Bearer,
            });
    r###"{
  "type": "http",
  "schema": "bearer",
  "bearerFormat": "JWT"
}"###
    }

    test_fn! {
        security_schema_correct_basic_auth:
        SecuritySchema::new(SecurityType::Http{schema: HttpAutheticationType::Basic, bearer_format: None});
        r###"{
  "type": "http",
  "schema": "basic"
}"###
    }

    test_fn! {
        security_schema_correct_digest_auth:
        SecuritySchema::new(SecurityType::Http{schema: HttpAutheticationType::Digest, bearer_format: None});
        r###"{
  "type": "http",
  "schema": "digest"
}"###
    }

    test_fn! {
        security_schema_correct_hoba_auth:
        SecuritySchema::new(SecurityType::Http{schema: HttpAutheticationType::Hoba, bearer_format: None});
        r###"{
  "type": "http",
  "schema": "hoba"
}"###
    }

    test_fn! {
        security_schema_correct_mutual_auth:
        SecuritySchema::new(SecurityType::Http{schema: HttpAutheticationType::Mutual, bearer_format: None});
        r###"{
  "type": "http",
  "schema": "mutual"
}"###
    }

    test_fn! {
        security_schema_correct_negotiate_auth:
        SecuritySchema::new(SecurityType::Http{schema: HttpAutheticationType::Negotiate, bearer_format: None});
        r###"{
  "type": "http",
  "schema": "negotiate"
}"###
    }

    test_fn! {
        security_schema_correct_oauth_auth:
        SecuritySchema::new(SecurityType::Http{schema: HttpAutheticationType::OAuth, bearer_format: None});
        r###"{
  "type": "http",
  "schema": "oauth"
}"###
    }

    test_fn! {
        security_schema_correct_scram_sha1_auth:
        SecuritySchema::new(SecurityType::Http{schema: HttpAutheticationType::ScramSha1, bearer_format: None});
        r###"{
  "type": "http",
  "schema": "scram-sha-1"
}"###
    }

    test_fn! {
        security_schema_correct_scram_sha256_auth:
        SecuritySchema::new(SecurityType::Http{schema: HttpAutheticationType::ScramSha256, bearer_format: None});
        r###"{
  "type": "http",
  "schema": "scram-sha-256"
}"###
    }

    test_fn! {
        security_schema_correct_api_key_cookie_auth:
        SecuritySchema::new(SecurityType::ApiKey{api_key_in: ApiKeyIn::Cookie , name: String::from("api_key")});
        r###"{
  "type": "apiKey",
  "name": "api_key",
  "in": "cookie"
}"###
    }

    test_fn! {
        security_schema_correct_api_key_header_auth:
        SecuritySchema::new(SecurityType::ApiKey{api_key_in: ApiKeyIn::Header , name: String::from("api_key")});
        r###"{
  "type": "apiKey",
  "name": "api_key",
  "in": "header"
}"###
    }

    test_fn! {
        security_schema_correct_api_key_query_auth:
        SecuritySchema::new(SecurityType::ApiKey{api_key_in: ApiKeyIn::Query , name: String::from("api_key")});
        r###"{
  "type": "apiKey",
  "name": "api_key",
  "in": "query"
}"###
    }

    test_fn! {
        security_schema_correct_open_id_connect_auth:
        SecuritySchema::new(SecurityType::OpenIdConnect{open_id_connect_url: String::from("http://localhost/openid")});
        r###"{
  "type": "openIdConnect",
  "openIdConnectUrl": "http://localhost/openid"
}"###
    }

    test_fn! {
        security_schema_correct_oauth2_implicit:
        SecuritySchema::new(SecurityType::OAuth2 {
            flows: Flows::new([Flow::Implicit(
                Implicit::new(
                    "http://localhost/auth/dialog",
                    HashMap::from([
                        ("edit:items".to_string(), "edit my items".to_string()),
                        ("read:items".to_string(), "read my items".to_string()
                    )]),
                ),
            )])
        });
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
  }
}"###
    }
}
