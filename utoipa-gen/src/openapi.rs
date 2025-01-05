use std::borrow::Cow;

use proc_macro2::Ident;
use syn::{
    bracketed, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token::{And, Comma},
    Attribute, Error, ExprPath, LitStr, Token, TypePath,
};

use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned, ToTokens};

use crate::{
    component::{features::Feature, ComponentSchema, Container, TypeTree},
    parse_utils,
    security_requirement::SecurityRequirementsAttr,
    Array, Diagnostics, ExternalDocs, ToTokensDiagnostics,
};
use crate::{path, OptionExt};

use self::info::Info;

mod info;

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct OpenApiAttr<'o> {
    info: Option<Info<'o>>,
    paths: Punctuated<ExprPath, Comma>,
    components: Components,
    modifiers: Punctuated<Modifier, Comma>,
    security: Option<Array<'static, SecurityRequirementsAttr>>,
    tags: Option<Array<'static, Tag>>,
    external_docs: Option<ExternalDocs>,
    servers: Punctuated<Server, Comma>,
    nested: Vec<NestOpenApi>,
}

impl<'o> OpenApiAttr<'o> {
    fn merge(mut self, other: OpenApiAttr<'o>) -> Self {
        if other.info.is_some() {
            self.info = other.info;
        }
        if !other.paths.is_empty() {
            self.paths = other.paths;
        }
        if !other.components.schemas.is_empty() {
            self.components.schemas = other.components.schemas;
        }
        if !other.components.responses.is_empty() {
            self.components.responses = other.components.responses;
        }
        if other.security.is_some() {
            self.security = other.security;
        }
        if other.tags.is_some() {
            self.tags = other.tags;
        }
        if other.external_docs.is_some() {
            self.external_docs = other.external_docs;
        }
        if !other.servers.is_empty() {
            self.servers = other.servers;
        }

        self
    }
}

pub fn parse_openapi_attrs(attrs: &[Attribute]) -> Result<Option<OpenApiAttr>, Error> {
    attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident("openapi"))
        .map(|attribute| attribute.parse_args::<OpenApiAttr>())
        .collect::<Result<Vec<_>, _>>()
        .map(|attrs| attrs.into_iter().reduce(|acc, item| acc.merge(item)))
}

impl Parse for OpenApiAttr<'_> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        const EXPECTED_ATTRIBUTE: &str =
            "unexpected attribute, expected any of: handlers, components, modifiers, security, tags, external_docs, servers, nest";
        let mut openapi = OpenApiAttr::default();

        while !input.is_empty() {
            let ident = input.parse::<Ident>().map_err(|error| {
                Error::new(error.span(), format!("{EXPECTED_ATTRIBUTE}, {error}"))
            })?;
            let attribute = &*ident.to_string();

            match attribute {
                "info" => {
                    let info_stream;
                    parenthesized!(info_stream in input);
                    openapi.info = Some(info_stream.parse()?)
                }
                "paths" => {
                    openapi.paths = parse_utils::parse_comma_separated_within_parenthesis(input)?;
                }
                "components" => {
                    openapi.components = input.parse()?;
                }
                "modifiers" => {
                    openapi.modifiers =
                        parse_utils::parse_comma_separated_within_parenthesis(input)?;
                }
                "security" => {
                    let security;
                    parenthesized!(security in input);
                    openapi.security = Some(parse_utils::parse_groups_collect(&security)?)
                }
                "tags" => {
                    let tags;
                    parenthesized!(tags in input);
                    openapi.tags = Some(parse_utils::parse_groups_collect(&tags)?);
                }
                "external_docs" => {
                    let external_docs;
                    parenthesized!(external_docs in input);
                    openapi.external_docs = Some(external_docs.parse()?);
                }
                "servers" => {
                    openapi.servers = parse_utils::parse_comma_separated_within_parenthesis(input)?;
                }
                "nest" => {
                    let nest;
                    parenthesized!(nest in input);
                    openapi.nested = parse_utils::parse_groups_collect(&nest)?;
                }
                _ => {
                    return Err(Error::new(ident.span(), EXPECTED_ATTRIBUTE));
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(openapi)
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct Schema(TypePath);

impl Schema {
    fn get_component(&self) -> Result<ComponentSchema, Diagnostics> {
        let ty = syn::Type::Path(self.0.clone());
        let type_tree = TypeTree::from_type(&ty)?;
        let generics = type_tree.get_path_generics()?;

        let container = Container {
            generics: &generics,
        };
        let component_schema = ComponentSchema::new(crate::component::ComponentSchemaProps {
            container: &container,
            type_tree: &type_tree,
            features: vec![Feature::Inline(true.into())],
            description: None,
        })?;

        Ok(component_schema)
    }
}

impl Parse for Schema {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse().map(Self)
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct Response(TypePath);

impl Parse for Response {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse().map(Self)
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct Modifier {
    and: And,
    ident: Ident,
}

impl ToTokens for Modifier {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let and = &self.and;
        let ident = &self.ident;
        tokens.extend(quote! {
            #and #ident
        })
    }
}

impl Parse for Modifier {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            and: input.parse()?,
            ident: input.parse()?,
        })
    }
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
struct Tag {
    name: parse_utils::LitStrOrExpr,
    description: Option<parse_utils::LitStrOrExpr>,
    external_docs: Option<ExternalDocs>,
}

impl Parse for Tag {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        const EXPECTED_ATTRIBUTE: &str =
            "unexpected token, expected any of: name, description, external_docs";

        let mut tag = Tag::default();

        while !input.is_empty() {
            let ident = input.parse::<Ident>().map_err(|error| {
                syn::Error::new(error.span(), format!("{EXPECTED_ATTRIBUTE}, {error}"))
            })?;
            let attribute_name = &*ident.to_string();

            match attribute_name {
                "name" => tag.name = parse_utils::parse_next_literal_str_or_expr(input)?,
                "description" => {
                    tag.description = Some(parse_utils::parse_next_literal_str_or_expr(input)?)
                }
                "external_docs" => {
                    let content;
                    parenthesized!(content in input);
                    tag.external_docs = Some(content.parse::<ExternalDocs>()?);
                }
                _ => return Err(syn::Error::new(ident.span(), EXPECTED_ATTRIBUTE)),
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(tag)
    }
}

impl ToTokens for Tag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        tokens.extend(quote! {
            utoipa::openapi::tag::TagBuilder::new().name(#name)
        });

        if let Some(ref description) = self.description {
            tokens.extend(quote! {
                .description(Some(#description))
            });
        }

        if let Some(ref external_docs) = self.external_docs {
            tokens.extend(quote! {
                .external_docs(Some(#external_docs))
            });
        }

        tokens.extend(quote! { .build() })
    }
}

// (url = "http:://url", description = "description", variables(...))
#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Server {
    url: String,
    description: Option<String>,
    variables: Punctuated<ServerVariable, Comma>,
}

impl Parse for Server {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let server_stream;
        parenthesized!(server_stream in input);
        let mut server = Server::default();
        while !server_stream.is_empty() {
            let ident = server_stream.parse::<Ident>()?;
            let attribute_name = &*ident.to_string();

            match attribute_name {
                "url" => {
                    server.url = parse_utils::parse_next(&server_stream, || server_stream.parse::<LitStr>())?.value()
                }
                "description" => {
                    server.description =
                        Some(parse_utils::parse_next(&server_stream, || server_stream.parse::<LitStr>())?.value())
                }
                "variables" => {
                    server.variables = parse_utils::parse_comma_separated_within_parenthesis(&server_stream)?
                }
                _ => {
                    return Err(Error::new(ident.span(), format!("unexpected attribute: {attribute_name}, expected one of: url, description, variables")))
                }
            }

            if !server_stream.is_empty() {
                server_stream.parse::<Comma>()?;
            }
        }

        Ok(server)
    }
}

impl ToTokens for Server {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let url = &self.url;
        let description = &self
            .description
            .as_ref()
            .map(|description| quote! { .description(Some(#description)) });

        let parameters = self
            .variables
            .iter()
            .map(|variable| {
                let name = &variable.name;
                let default_value = &variable.default;
                let description = &variable
                    .description
                    .as_ref()
                    .map(|description| quote! { .description(Some(#description)) });
                let enum_values = &variable.enum_values.as_ref().map(|enum_values| {
                    let enum_values = enum_values.iter().collect::<Array<&LitStr>>();

                    quote! { .enum_values(Some(#enum_values)) }
                });

                quote! {
                    .parameter(#name, utoipa::openapi::server::ServerVariableBuilder::new()
                        .default_value(#default_value)
                        #description
                        #enum_values
                    )
                }
            })
            .collect::<TokenStream>();

        tokens.extend(quote! {
            utoipa::openapi::server::ServerBuilder::new()
                .url(#url)
                #description
                #parameters
                .build()
        })
    }
}

// ("username" = (default = "demo", description = "This is default username for the API")),
// ("port" = (enum_values = (8080, 5000, 4545)))
#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
struct ServerVariable {
    name: String,
    default: String,
    description: Option<String>,
    enum_values: Option<Punctuated<LitStr, Comma>>,
}

impl Parse for ServerVariable {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let variable_stream;
        parenthesized!(variable_stream in input);
        let mut server_variable = ServerVariable {
            name: variable_stream.parse::<LitStr>()?.value(),
            ..ServerVariable::default()
        };

        variable_stream.parse::<Token![=]>()?;
        let content;
        parenthesized!(content in variable_stream);

        while !content.is_empty() {
            let ident = content.parse::<Ident>()?;
            let attribute_name = &*ident.to_string();

            match attribute_name {
                "default" => {
                    server_variable.default =
                        parse_utils::parse_next(&content, || content.parse::<LitStr>())?.value()
                }
                "description" => {
                    server_variable.description =
                        Some(parse_utils::parse_next(&content, || content.parse::<LitStr>())?.value())
                }
                "enum_values" => {
                    server_variable.enum_values =
                        Some(parse_utils::parse_comma_separated_within_parenthesis(&content)?)
                }
                _ => {
                    return Err(Error::new(ident.span(), format!( "unexpected attribute: {attribute_name}, expected one of: default, description, enum_values")))
                }
            }

            if !content.is_empty() {
                content.parse::<Comma>()?;
            }
        }

        Ok(server_variable)
    }
}

pub(crate) struct OpenApi<'o>(pub Option<OpenApiAttr<'o>>, pub Ident);

impl OpenApi<'_> {
    fn nested_tokens(&self) -> Option<TokenStream> {
        let nested = self.0.as_ref().map(|openapi| &openapi.nested)?;
        let nest_tokens = nested.iter()
                .map(|item| {
                    let path = &item.path;
                    let nest_api = &item
                        .open_api
                        .as_ref()
                        .expect("type path of nested api is mandatory");
                    let nest_api_ident = &nest_api
                        .path
                        .segments
                        .last()
                        .expect("nest api must have at least one segment")
                        .ident;
                    let nest_api_config = format_ident!("{}Config", nest_api_ident.to_string());

                    let module_path = nest_api
                        .path
                        .segments
                        .iter()
                        .take(nest_api.path.segments.len() - 1)
                        .map(|segment| segment.ident.to_string())
                        .collect::<Vec<_>>()
                        .join("::");
                    let tags = &item.tags.iter().collect::<Array<_>>();

                    let span = nest_api.span();
                    quote_spanned! {span=>
                        .nest(#path, {
                            #[allow(non_camel_case_types)]
                            struct #nest_api_config;
                            impl utoipa::__dev::NestedApiConfig for #nest_api_config {
                                fn config() -> (utoipa::openapi::OpenApi, Vec<&'static str>, &'static str) {
                                    let api = <#nest_api as utoipa::OpenApi>::openapi();

                                    (api, #tags.into(), #module_path)
                                }
                            }
                            <#nest_api_config as utoipa::OpenApi>::openapi()
                        })
                    }
                })
                .collect::<TokenStream>();

        if nest_tokens.is_empty() {
            None
        } else {
            Some(nest_tokens)
        }
    }
}

impl ToTokensDiagnostics for OpenApi<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) -> Result<(), Diagnostics> {
        let OpenApi(attributes, ident) = self;

        let info = Info::merge_with_env_args(
            attributes
                .as_ref()
                .and_then(|attributes| attributes.info.clone()),
        );

        let components = attributes
            .as_ref()
            .map_try(|attributes| attributes.components.try_to_token_stream())?
            .and_then(|tokens| {
                if !tokens.is_empty() {
                    Some(quote! { .components(Some(#tokens)) })
                } else {
                    None
                }
            });

        let Paths(path_items, handlers) =
            impl_paths(attributes.as_ref().map(|attributes| &attributes.paths));

        let handler_schemas = handlers.iter().fold(
            quote! {
                    let components = openapi.components.get_or_insert(utoipa::openapi::Components::new());
                    let mut schemas = Vec::<(String, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>)>::new();
            },
            |mut handler_schemas, (usage, ..)| {
                handler_schemas.extend(quote! {
                    <#usage as utoipa::__dev::SchemaReferences>::schemas(&mut schemas);
                });

                handler_schemas
            },
        );

        let securities = attributes
            .as_ref()
            .and_then(|openapi_attributes| openapi_attributes.security.as_ref())
            .map(|securities| {
                quote! {
                    .security(Some(#securities))
                }
            });
        let tags = attributes
            .as_ref()
            .and_then(|attributes| attributes.tags.as_ref())
            .map(|tags| {
                quote! {
                    .tags(Some(#tags))
                }
            });
        let external_docs = attributes
            .as_ref()
            .and_then(|attributes| attributes.external_docs.as_ref())
            .map(|external_docs| {
                quote! {
                    .external_docs(Some(#external_docs))
                }
            });

        let servers = match attributes.as_ref().map(|attributes| &attributes.servers) {
            Some(servers) if !servers.is_empty() => {
                let servers = servers.iter().collect::<Array<&Server>>();
                Some(quote! { .servers(Some(#servers)) })
            }
            _ => None,
        };

        let modifiers_tokens = attributes
            .as_ref()
            .map(|attributes| &attributes.modifiers)
            .map(|modifiers| {
                let modifiers_len = modifiers.len();

                quote! {
                    let _mods: [&dyn utoipa::Modify; #modifiers_len] = [#modifiers];
                    _mods.iter().for_each(|modifier| modifier.modify(&mut openapi));
                }
            });

        let nested_tokens = self
            .nested_tokens()
            .map(|tokens| quote! {openapi = openapi #tokens;});
        tokens.extend(quote! {
            impl utoipa::OpenApi for #ident {
                fn openapi() -> utoipa::openapi::OpenApi {
                    use utoipa::{ToSchema, Path};
                    let mut openapi = utoipa::openapi::OpenApiBuilder::new()
                        .info(#info)
                        .paths({
                            #path_items
                        })
                        #components
                        #securities
                        #tags
                        #servers
                        #external_docs
                        .build();
                    #handler_schemas
                    components.schemas.extend(schemas);
                    #nested_tokens

                    #modifiers_tokens

                    openapi
                }
            }
        });

        Ok(())
    }
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
struct Components {
    schemas: Vec<Schema>,
    responses: Vec<Response>,
}

impl Parse for Components {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);
        const EXPECTED_ATTRIBUTE: &str =
            "unexpected attribute. expected one of: schemas, responses";

        let mut schemas: Vec<Schema> = Vec::new();
        let mut responses: Vec<Response> = Vec::new();

        while !content.is_empty() {
            let ident = content.parse::<Ident>().map_err(|error| {
                Error::new(error.span(), format!("{EXPECTED_ATTRIBUTE}, {error}"))
            })?;
            let attribute = &*ident.to_string();

            match attribute {
                "schemas" => schemas.append(
                    &mut parse_utils::parse_comma_separated_within_parenthesis(&content)?
                        .into_iter()
                        .collect(),
                ),
                "responses" => responses.append(
                    &mut parse_utils::parse_comma_separated_within_parenthesis(&content)?
                        .into_iter()
                        .collect(),
                ),
                _ => return Err(syn::Error::new(ident.span(), EXPECTED_ATTRIBUTE)),
            }

            if !content.is_empty() {
                content.parse::<Token![,]>()?;
            }
        }

        Ok(Self { schemas, responses })
    }
}

impl ToTokensDiagnostics for Components {
    fn to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostics> {
        if self.schemas.is_empty() && self.responses.is_empty() {
            return Ok(());
        }

        let builder_tokens = self
            .schemas
            .iter()
            .map(|schema| match schema.get_component() {
                Ok(component_schema) => Ok((component_schema, &schema.0)),
                Err(diagnostics) => Err(diagnostics),
            })
            .collect::<Result<Vec<(ComponentSchema, &TypePath)>, Diagnostics>>()?
            .into_iter()
            .fold(
                quote! { utoipa::openapi::ComponentsBuilder::new() },
                |mut components, (component_schema, type_path)| {
                    let schema = component_schema.to_token_stream();
                    let name = &component_schema.name_tokens;

                    components.extend(quote! { .schemas_from_iter( {
                        let mut schemas = Vec::<(String, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>)>::new();
                        <#type_path as utoipa::ToSchema>::schemas(&mut schemas);
                        schemas
                    } )});
                    components.extend(quote! { .schema(#name, {
                        let mut generics = Vec::<utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>>::new();
                        #schema
                    }) });

                    components
                },
            );

        let builder_tokens =
            self.responses
                .iter()
                .fold(builder_tokens, |mut builder_tokens, responses| {
                    let Response(path) = responses;

                    builder_tokens.extend(quote_spanned! {path.span() =>
                        .response_from::<#path>()
                    });
                    builder_tokens
                });

        tokens.extend(quote! { #builder_tokens.build() });

        Ok(())
    }
}

struct Paths(TokenStream, Vec<(ExprPath, String, Ident)>);

fn impl_paths(handler_paths: Option<&Punctuated<ExprPath, Comma>>) -> Paths {
    let handlers = handler_paths
        .into_iter()
        .flatten()
        .map(|handler| {
            let segments = handler.path.segments.iter().collect::<Vec<_>>();
            let handler_config_name = segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>()
                .join("_");
            let handler_fn = &segments.last().unwrap().ident;
            let handler_ident = path::format_path_ident(Cow::Borrowed(handler_fn));
            let handler_ident_config = format_ident!("{}_config", handler_config_name);

            let tag = segments
                .iter()
                .take(segments.len() - 1)
                .map(|part| part.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            let usage = syn::parse_str::<ExprPath>(
                &vec![
                    if tag.is_empty() { None } else { Some(&*tag) },
                    Some(&*handler_ident.as_ref().to_string()),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join("::"),
            )
            .unwrap();
            (usage, tag, handler_ident_config)
        })
        .collect::<Vec<_>>();

    let handlers_impls = handlers
        .iter()
        .map(|(usage, tag, handler_ident_nested)| {
            quote! {
                #[allow(non_camel_case_types)]
                struct #handler_ident_nested;
                #[allow(non_camel_case_types)]
                impl utoipa::__dev::PathConfig for #handler_ident_nested {
                    fn path() -> String {
                        #usage::path()
                    }
                    fn methods() -> Vec<utoipa::openapi::path::HttpMethod> {
                        #usage::methods()
                    }
                    fn tags_and_operation() -> (Vec<&'static str>, utoipa::openapi::path::Operation) {
                        let item = #usage::operation();
                        let mut tags = <#usage as utoipa::__dev::Tags>::tags();
                        if !#tag.is_empty() && tags.is_empty() {
                            tags.push(#tag);
                        }

                        (tags, item)
                    }
                }
            }
        })
        .collect::<TokenStream>();

    let tokens = handler_paths.into_iter().flatten().fold(
        quote! { #handlers_impls utoipa::openapi::path::PathsBuilder::new() },
        |mut paths, handler| {
            let segments = handler.path.segments.iter().collect::<Vec<_>>();
            let handler_config_name = segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>()
                .join("_");
            let handler_ident_config = format_ident!("{}_config", handler_config_name);

            paths.extend(quote! {
                .path_from::<#handler_ident_config>()
            });

            paths
        },
    );

    Paths(tokens, handlers)
}

/// (path = "/nest/path", api = NestApi, tags = ["tag1", "tag2"])
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Default)]
struct NestOpenApi {
    path: parse_utils::LitStrOrExpr,
    open_api: Option<TypePath>,
    tags: Punctuated<parse_utils::LitStrOrExpr, Comma>,
}

impl Parse for NestOpenApi {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        const ERROR_MESSAGE: &str = "unexpected identifier, expected any of: path, api, tags";
        let mut nest = NestOpenApi::default();

        while !input.is_empty() {
            let ident = input.parse::<Ident>().map_err(|error| {
                syn::Error::new(error.span(), format!("{ERROR_MESSAGE}: {error}"))
            })?;

            match &*ident.to_string() {
                "path" => nest.path = parse_utils::parse_next_literal_str_or_expr(input)?,
                "api" => nest.open_api = Some(parse_utils::parse_next(input, || input.parse())?),
                "tags" => {
                    nest.tags = parse_utils::parse_next(input, || {
                        let tags;
                        bracketed!(tags in input);
                        Punctuated::parse_terminated(&tags)
                    })?;
                }
                _ => return Err(syn::Error::new(ident.span(), ERROR_MESSAGE)),
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        if nest.path.is_empty_litstr() {
            return Err(syn::Error::new(
                input.span(),
                "`path = ...` argument is mandatory for nest(...) statement",
            ));
        }
        if nest.open_api.is_none() {
            return Err(syn::Error::new(
                input.span(),
                "`api = ...` argument is mandatory for nest(...) statement",
            ));
        }

        Ok(nest)
    }
}
