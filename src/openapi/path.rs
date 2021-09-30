use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{
    request_body::RequestBody,
    response::{Response, Responses},
    Deprecated, ExternalDocs, Required, Security, Server,
};

#[non_exhaustive]
#[derive(Default)]
pub struct Paths {
    inner: Vec<(String, PathItem)>,
}

impl Paths {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn append<S: AsRef<str>>(mut self, path: S, item: PathItem) -> Self {
        self.inner.push((path.as_ref().to_string(), item));

        self
    }

    pub fn to_map(self) -> HashMap<String, PathItem> {
        self.collect()
    }
}

impl Iterator for Paths {
    type Item = (String, PathItem);

    fn next(&mut self) -> Option<Self::Item> {
        if self.inner.is_empty() {
            None
        } else {
            Some(self.inner.remove(0))
        }
    }
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PathItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<Operation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub put: Option<Operation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<Operation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<Operation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Operation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<Operation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<Operation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<Operation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<Vec<Server>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<Parameter>>,
}

impl PathItem {
    pub fn new_get(operation: Operation) -> Self {
        Self {
            get: Some(operation),
            ..Default::default()
        }
    }

    pub fn new_put(operation: Operation) -> Self {
        Self {
            put: Some(operation),
            ..Default::default()
        }
    }

    pub fn new_post(operation: Operation) -> Self {
        Self {
            post: Some(operation),
            ..Default::default()
        }
    }

    pub fn new_delete(operation: Operation) -> Self {
        Self {
            delete: Some(operation),
            ..Default::default()
        }
    }

    pub fn new_options(operation: Operation) -> Self {
        Self {
            options: Some(operation),
            ..Default::default()
        }
    }

    pub fn new_head(operation: Operation) -> Self {
        Self {
            head: Some(operation),
            ..Default::default()
        }
    }

    pub fn new_patch(operation: Operation) -> Self {
        Self {
            patch: Some(operation),
            ..Default::default()
        }
    }

    pub fn new_trace(operation: Operation) -> Self {
        Self {
            trace: Some(operation),
            ..Default::default()
        }
    }

    pub fn with_summary<S: AsRef<str>>(mut self, summary: S) -> Self {
        self.summary = Some(summary.as_ref().to_string());

        self
    }

    pub fn with_description<S: AsRef<str>>(mut self, description: S) -> Self {
        self.description = Some(description.as_ref().to_string());

        self
    }

    pub fn with_servers<I: IntoIterator<Item = Server>>(mut self, servers: I) -> Self {
        self.servers = Some(servers.into_iter().collect());

        self
    }

    pub fn with_parameters<I: IntoIterator<Item = Parameter>>(mut self, parameters: I) -> Self {
        self.parameters = Some(parameters.into_iter().collect());

        self
    }
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<ExternalDocs>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<Parameter>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_body: Option<RequestBody>,

    pub responses: Responses,

    // TODO
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callbacks: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<Deprecated>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<Security>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<Vec<Server>>,
}

impl Operation {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_tags<I: IntoIterator<Item = String>>(mut self, tags: I) -> Self {
        self.tags = Some(tags.into_iter().collect());

        self
    }

    pub fn with_tag<S: AsRef<str>>(mut self, tag: S) -> Self {
        self.tags.as_mut().unwrap().push(tag.as_ref().to_string());

        self
    }

    pub fn with_summary<S: AsRef<str>>(mut self, summary: S) -> Self {
        self.summary = Some(summary.as_ref().to_string());

        self
    }

    pub fn with_description<S: AsRef<str>>(mut self, description: S) -> Self {
        self.description = Some(description.as_ref().to_string());

        self
    }

    pub fn with_operation_id<S: AsRef<str>>(mut self, operation_id: S) -> Self {
        self.operation_id = Some(operation_id.as_ref().to_string());

        self
    }

    pub fn with_parameters<I: IntoIterator<Item = Parameter>>(mut self, parameters: I) -> Self {
        self.parameters = Some(parameters.into_iter().collect());

        self
    }

    pub fn with_parameter(mut self, parameter: Parameter) -> Self {
        self.parameters.as_mut().unwrap().push(parameter);

        self
    }

    pub fn with_request_body(mut self, request_body: RequestBody) -> Self {
        self.request_body = Some(request_body);

        self
    }

    pub fn with_responses(mut self, responses: Responses) -> Self {
        self.responses = responses;

        self
    }

    pub fn with_response<S: AsRef<str>>(mut self, code: S, response: Response) -> Self {
        self.responses = self.responses.with_response(code, response);

        self
    }

    pub fn with_securities<I: IntoIterator<Item = Security>>(mut self, securities: I) -> Self {
        self.security = Some(securities.into_iter().collect());

        self
    }

    pub fn with_security(mut self, security: Security) -> Self {
        self.security.as_mut().unwrap().push(security);

        self
    }

    pub fn with_servers<I: IntoIterator<Item = Server>>(mut self, servers: I) -> Self {
        self.servers = Some(servers.into_iter().collect());

        self
    }

    pub fn with_server(mut self, server: Server) -> Self {
        self.servers.as_mut().unwrap().push(server);

        self
    }
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    pub name: String,

    #[serde(rename = "in")]
    pub parameter_in: ParameterIn,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    pub required: Required,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<Deprecated>,
    // pub allow_empty_value: bool, this is going to be removed from further open api spec releases
}

impl Parameter {
    pub fn new<S: AsRef<str>>(name: S) -> Self {
        Self {
            name: name.as_ref().to_string(),
            required: Required::True,
            ..Default::default()
        }
    }

    pub fn with_in(mut self, parameter_in: ParameterIn) -> Self {
        self.parameter_in = parameter_in;

        self
    }

    pub fn with_required(mut self, required: Required) -> Self {
        self.required = required;
        // required must be true, if parameter_in is Path
        if self.parameter_in == ParameterIn::Path {
            self.required = Required::True;
        }

        self
    }

    pub fn with_description<S: AsRef<str>>(mut self, description: S) -> Self {
        self.description = Some(description.as_ref().to_string());

        self
    }

    pub fn with_deprecated(mut self, deprecated: Deprecated) -> Self {
        self.deprecated = Some(deprecated);

        self
    }
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub enum ParameterIn {
    Query,
    Path,
    Header,
    Cookie,
}

impl Default for ParameterIn {
    fn default() -> Self {
        Self::Path
    }
}

impl Serialize for ParameterIn {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *self {
            ParameterIn::Query => serializer.serialize_str("query"),
            ParameterIn::Path => serializer.serialize_str("path"),
            ParameterIn::Header => serializer.serialize_str("header"),
            ParameterIn::Cookie => serializer.serialize_str("cookie"),
        }
    }
}
