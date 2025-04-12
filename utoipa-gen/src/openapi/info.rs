use std::borrow::Cow;
use std::io;

use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::parse::Parse;
use syn::token::Comma;
use syn::{parenthesized, Error, LitStr};

use crate::parse_utils::{self, LitStrOrExpr};

#[derive(Default, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub(super) struct Info<'i> {
    title: Option<LitStrOrExpr>,
    version: Option<LitStrOrExpr>,
    description: Option<LitStrOrExpr>,
    terms_of_service: Option<LitStrOrExpr>,
    license: Option<License<'i>>,
    contact: Option<Contact<'i>>,
}

impl Info<'_> {
    /// Construct new [`Info`] from _`cargo`_ env variables such as
    /// * `CARGO_PGK_NAME`
    /// * `CARGO_PGK_VERSION`
    /// * `CARGO_PGK_DESCRIPTION`
    /// * `CARGO_PGK_AUTHORS`
    /// * `CARGO_PGK_LICENSE`
    pub fn from_env() -> Self {
        let name = std::env::var("CARGO_PKG_NAME").ok();
        let version = std::env::var("CARGO_PKG_VERSION").ok();
        let description = std::env::var("CARGO_PKG_DESCRIPTION").ok();
        let contact = std::env::var("CARGO_PKG_AUTHORS")
            .ok()
            .and_then(|authors| Contact::try_from(authors).ok())
            .and_then(|contact| {
                if contact.name.is_none() && contact.email.is_none() && contact.url.is_none() {
                    None
                } else {
                    Some(contact)
                }
            });
        let license = std::env::var("CARGO_PKG_LICENSE")
            .ok()
            .map(|spdx_expr| License {
                name: Cow::Owned(spdx_expr.clone()),
                // CARGO_PKG_LICENSE contains an SPDX expression as described in the Cargo Book.
                // It can be set to `info.license.identifier`.
                identifier: Cow::Owned(spdx_expr),
                ..Default::default()
            });

        Info {
            title: name.map(|name| name.into()),
            version: version.map(|version| version.into()),
            description: description.map(|description| description.into()),
            contact,
            license,
            ..Default::default()
        }
    }

    /// Merge given info arguments to [`Info`] created from `CARGO_*` env arguments.
    pub fn merge_with_env_args(info: Option<Info>) -> Info {
        let mut from_env = Info::from_env();
        if let Some(info) = info {
            if info.title.is_some() {
                from_env.title = info.title;
            }

            if info.terms_of_service.is_some() {
                from_env.terms_of_service = info.terms_of_service;
            }

            if info.description.is_some() {
                from_env.description = info.description;
            }

            if info.license.is_some() {
                from_env.license = info.license;
            }

            if info.contact.is_some() {
                from_env.contact = info.contact;
            }

            if info.version.is_some() {
                from_env.version = info.version;
            }
        }

        from_env
    }
}

impl Parse for Info<'_> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut info = Info::default();

        while !input.is_empty() {
            let ident = input.parse::<Ident>()?;
            let attribute_name = &*ident.to_string();

            match attribute_name {
                "title" => {
                    info.title = Some(parse_utils::parse_next(input, || {
                        input.parse::<LitStrOrExpr>()
                    })?)
                }
                "version" => {
                    info.version = Some(parse_utils::parse_next(input, || {
                        input.parse::<LitStrOrExpr>()
                    })?)
                }
                "description" => {
                    info.description = Some(parse_utils::parse_next(input, || {
                        input.parse::<LitStrOrExpr>()
                    })?)
                }
                "terms_of_service" => {
                    info.terms_of_service = Some(parse_utils::parse_next(input, || {
                        input.parse::<LitStrOrExpr>()
                    })?)
                }
                "license" => {
                    let license_stream;
                    parenthesized!(license_stream in input);
                    info.license = Some(license_stream.parse()?)
                }
                "contact" => {
                    let contact_stream;
                    parenthesized!(contact_stream in input);
                    info.contact = Some(contact_stream.parse()?)
                }
                _ => {
                    return Err(Error::new(ident.span(), format!("unexpected attribute: {attribute_name}, expected one of: title, terms_of_service, version, description, license, contact")));
                }
            }
            if !input.is_empty() {
                input.parse::<Comma>()?;
            }
        }

        Ok(info)
    }
}

impl ToTokens for Info<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let title = self.title.as_ref().map(|title| quote! { .title(#title) });
        let version = self
            .version
            .as_ref()
            .map(|version| quote! { .version(#version) });
        let terms_of_service = self
            .terms_of_service
            .as_ref()
            .map(|terms_of_service| quote! {.terms_of_service(Some(#terms_of_service))});
        let description = self
            .description
            .as_ref()
            .map(|description| quote! { .description(Some(#description)) });
        let license = self
            .license
            .as_ref()
            .map(|license| quote! { .license(Some(#license)) });
        let contact = self
            .contact
            .as_ref()
            .map(|contact| quote! { .contact(Some(#contact)) });

        tokens.extend(quote! {
            utoipa::openapi::InfoBuilder::new()
                #title
                #version
                #terms_of_service
                #description
                #license
                #contact
        })
    }
}

#[derive(Default, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub(super) struct License<'l> {
    name: Cow<'l, str>,
    url: Option<Cow<'l, str>>,
    identifier: Cow<'l, str>,
}

impl Parse for License<'_> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut license = License::default();

        while !input.is_empty() {
            let ident = input.parse::<Ident>()?;
            let attribute_name = &*ident.to_string();

            match attribute_name {
                "name" => {
                    license.name = Cow::Owned(
                        parse_utils::parse_next(input, || input.parse::<LitStr>())?.value(),
                    )
                }
                "url" => {
                    license.url = Some(Cow::Owned(
                        parse_utils::parse_next(input, || input.parse::<LitStr>())?.value(),
                    ))
                }
                "identifier" => {
                    license.identifier = Cow::Owned(
                        parse_utils::parse_next(input, || input.parse::<LitStr>())?.value(),
                    )
                }
                _ => {
                    return Err(Error::new(
                        ident.span(),
                        format!(
                            "unexpected attribute: {attribute_name}, expected one of: name, url"
                        ),
                    ));
                }
            }
            if !input.is_empty() {
                input.parse::<Comma>()?;
            }
        }

        Ok(license)
    }
}

impl ToTokens for License<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let name = &self.name;
        let url = self.url.as_ref().map(|url| quote! { .url(Some(#url))});
        let identifier = if !self.identifier.is_empty() {
            let identifier = self.identifier.as_ref();
            quote! { .identifier(Some(#identifier))}
        } else {
            TokenStream2::new()
        };

        tokens.extend(quote! {
            utoipa::openapi::info::LicenseBuilder::new()
                .name(#name)
                #url
                #identifier
                .build()
        })
    }
}

impl From<String> for License<'_> {
    fn from(string: String) -> Self {
        License {
            name: Cow::Owned(string),
            ..Default::default()
        }
    }
}

#[derive(Default, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub(super) struct Contact<'c> {
    name: Option<Cow<'c, str>>,
    email: Option<Cow<'c, str>>,
    url: Option<Cow<'c, str>>,
}

impl Parse for Contact<'_> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut contact = Contact::default();

        while !input.is_empty() {
            let ident = input.parse::<Ident>()?;
            let attribute_name = &*ident.to_string();

            match attribute_name {
                "name" => {
                    contact.name = Some(Cow::Owned(
                        parse_utils::parse_next(input, || input.parse::<LitStr>())?.value(),
                    ))
                }
                "email" => {
                    contact.email = Some(Cow::Owned(
                        parse_utils::parse_next(input, || input.parse::<LitStr>())?.value(),
                    ))
                }
                "url" => {
                    contact.url = Some(Cow::Owned(
                        parse_utils::parse_next(input, || input.parse::<LitStr>())?.value(),
                    ))
                }
                _ => {
                    return Err(Error::new(
                        ident.span(),
                        format!("unexpected attribute: {attribute_name}, expected one of: name, email, url"),
                    ));
                }
            }
            if !input.is_empty() {
                input.parse::<Comma>()?;
            }
        }

        Ok(contact)
    }
}

impl ToTokens for Contact<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let name = self.name.as_ref().map(|name| quote! { .name(Some(#name)) });
        let email = self
            .email
            .as_ref()
            .map(|email| quote! { .email(Some(#email)) });
        let url = self.url.as_ref().map(|url| quote! { .url(Some(#url)) });

        tokens.extend(quote! {
            utoipa::openapi::info::ContactBuilder::new()
                #name
                #email
                #url
                .build()
        })
    }
}

impl TryFrom<String> for Contact<'_> {
    type Error = io::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if let Some((name, email)) = get_parsed_author(value.split(':').next()) {
            let non_empty = |value: &str| -> Option<Cow<'static, str>> {
                if !value.is_empty() {
                    Some(Cow::Owned(value.to_string()))
                } else {
                    None
                }
            };
            Ok(Contact {
                name: non_empty(name),
                email: non_empty(email),
                ..Default::default()
            })
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("invalid contact: {value}"),
            ))
        }
    }
}

fn get_parsed_author(author: Option<&str>) -> Option<(&str, &str)> {
    author.map(|author| {
        let mut author_iter = author.split('<');

        let name = author_iter.next().unwrap_or_default();
        let mut email = author_iter.next().unwrap_or_default();
        if !email.is_empty() {
            email = &email[..email.len() - 1];
        }

        (name.trim_end(), email)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_author_with_email_success() {
        let author = "Tessu Tester <tessu@steps.com>";

        if let Some((name, email)) = get_parsed_author(Some(author)) {
            assert_eq!(
                name, "Tessu Tester",
                "expected name {} != {}",
                "Tessu Tester", name
            );
            assert_eq!(
                email, "tessu@steps.com",
                "expected email {} != {}",
                "tessu@steps.com", email
            );
        } else {
            panic!("Expected Some(Tessu Tester, tessu@steps.com), but was none")
        }
    }

    #[test]
    fn parse_author_only_name() {
        let author = "Tessu Tester";

        if let Some((name, email)) = get_parsed_author(Some(author)) {
            assert_eq!(
                name, "Tessu Tester",
                "expected name {} != {}",
                "Tessu Tester", name
            );
            assert_eq!(email, "", "expected email {} != {}", "", email);
        } else {
            panic!("Expected Some(Tessu Tester, ), but was none")
        }
    }

    #[test]
    fn contact_from_only_name() {
        let author = "Suzy Lin";
        let contanct = Contact::try_from(author.to_string()).unwrap();

        assert!(contanct.name.is_some(), "Suzy should have name");
        assert!(contanct.email.is_none(), "Suzy should not have email");
    }

    #[test]
    fn contact_from_name_and_email() {
        let author = "Suzy Lin <suzy@lin.com>";
        let contanct = Contact::try_from(author.to_string()).unwrap();

        assert!(contanct.name.is_some(), "Suzy should have name");
        assert!(contanct.email.is_some(), "Suzy should have email");
    }

    #[test]
    fn contact_from_empty() {
        let author = "";
        let contact = Contact::try_from(author.to_string()).unwrap();

        assert!(contact.name.is_none(), "Contact name should be empty");
        assert!(contact.email.is_none(), "Contact email should be empty");
    }

    #[test]
    fn info_from_env() {
        let info = Info::from_env();

        match info.title {
            Some(LitStrOrExpr::LitStr(title)) => assert_eq!(title.value(), env!("CARGO_PKG_NAME")),
            _ => panic!(),
        }

        match info.version {
            Some(LitStrOrExpr::LitStr(version)) => {
                assert_eq!(version.value(), env!("CARGO_PKG_VERSION"))
            }
            _ => panic!(),
        }

        match info.description {
            Some(LitStrOrExpr::LitStr(description)) => {
                assert_eq!(description.value(), env!("CARGO_PKG_DESCRIPTION"))
            }
            _ => panic!(),
        }

        assert!(matches!(info.terms_of_service, None));

        match info.license {
            Some(license) => {
                assert_eq!(license.name, env!("CARGO_PKG_LICENSE"));
                assert_eq!(license.identifier, env!("CARGO_PKG_LICENSE"));
                assert_eq!(license.url, None);
            }
            None => panic!(),
        }
    }
}
