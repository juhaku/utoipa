use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{
    request_body::RequestBody,
    response::{Response, Responses},
    Component, Deprecated, ExternalDocs, Required, SecurityRequirement, Server,
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

    pub fn to_map(self) -> BTreeMap<String, PathItem> {
        self.fold(BTreeMap::new(), |mut acc, (path, path_item)| {
            if let Some(item) = acc.get_mut(&path) {
                item.merge_operations(path_item);
            } else {
                acc.insert(path, path_item);
            }

            acc
        })
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
#[derive(Serialize, Deserialize, Default, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct PathItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<Vec<Server>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<Parameter>>,

    #[serde(flatten)]
    pub operations: BTreeMap<PathItemType, Operation>,
}

impl PathItem {
    pub fn new(path_item_type: PathItemType, operation: Operation) -> Self {
        let mut operations = BTreeMap::new();

        operations.insert(path_item_type, operation);

        Self {
            operations,
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

    fn merge_operations(&mut self, mut another: PathItem) {
        self.operations.append(&mut another.operations);
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Clone)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum PathItemType {
    Get,
    Post,
    Put,
    Delete,
    Options,
    Head,
    Patch,
    Trace,
    Connect,
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
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
    pub security: Option<Vec<SecurityRequirement>>,

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
        let tag_string = tag.as_ref().to_string();
        match self.tags {
            Some(ref mut tags) => tags.push(tag_string),
            None => {
                self.tags = Some(vec![tag_string]);
            }
        }

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
        match self.parameters {
            Some(ref mut parameters) => parameters.push(parameter),
            None => {
                self.parameters = Some(vec![parameter]);
            }
        }

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

    pub fn with_response<S: Into<String>>(mut self, code: S, response: Response) -> Self {
        self.responses.responses.insert(code.into(), response);

        self
    }

    pub fn with_deprecated(mut self, deprecated: Deprecated) -> Self {
        self.deprecated = Some(deprecated);

        self
    }

    /// Add list of [`SecurityRequirement`]s that are globally available for all operations.
    pub fn with_securities<I: IntoIterator<Item = SecurityRequirement>>(
        mut self,
        securities: I,
    ) -> Self {
        self.security = Some(securities.into_iter().collect());

        self
    }

    /// Add [`SecurityRequirement`] that is globally available for all operations.
    pub fn with_security(mut self, security: SecurityRequirement) -> Self {
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
#[derive(Serialize, Deserialize, Default, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<Component>,
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

    pub fn with_schema<I: Into<Component>>(mut self, component: I) -> Self {
        self.schema = Some(component.into());

        self
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "debug", derive(Debug))]
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
