use std::collections::HashMap;

use serde::Serialize;

const END_MARKER: &str = "//</editor-fold>";

// https://github.com/swagger-api/swagger-ui/blob/master/docs/usage/oauth2.md
#[derive(Default, Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope_separator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_query_string_params: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_basic_authentication_with_access_code_grant: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_pkce_with_authorization_code_grant: Option<bool>,
}

pub(crate) fn format_swagger_config(config: &Config, file: String) -> serde_json::Result<String> {
    let init_string = format!(
        "{}\nui.initOAuth({});",
        END_MARKER,
        serde_json::to_string_pretty(config)?
    );
    Ok(file.replace(END_MARKER, &init_string))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_CONTENT: &str = r###""
    //<editor-fold desc=\"Changeable Configuration Block\">
    window.ui = SwaggerUIBundle({
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
    });
    //</editor-fold>
    ""###;

    #[test]
    fn format_swagger_config_oauth() {
        let config = Config {
            client_id: Some(String::from("my-special-client")),
            ..Default::default()
        };
        let file = super::format_swagger_config(&config, TEST_CONTENT.to_string()).unwrap();

        let expected = r#"
ui.initOAuth({
  "clientId": "my-special-client"
});"#;
        assert!(
            file.contains(expected),
            "expected file to contain {}, was {}",
            expected,
            file
        )
    }
}
