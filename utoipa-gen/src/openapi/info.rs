use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub(crate) fn impl_info() -> TokenStream2 {
    let name = std::env::var("CARGO_PKG_NAME").unwrap_or_default();
    let version = std::env::var("CARGO_PKG_VERSION").unwrap_or_default();
    let description = std::env::var("CARGO_PKG_DESCRIPTION").unwrap_or_default();
    let authors = std::env::var("CARGO_PKG_AUTHORS").unwrap_or_default();
    let license = std::env::var("CARGO_PKG_LICENSE").unwrap_or_default();

    let contact = get_contact(&authors);

    quote! {
        utoipa::openapi::InfoBuilder::new()
            .title(#name)
            .version(#version)
            .description(Some(#description))
            .license(Some(utoipa::openapi::License::new(#license)))
            .contact(Some(#contact))
            .build()
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

fn get_contact(authors: &str) -> TokenStream2 {
    if let Some((name, email)) = get_parsed_author(authors.split(':').into_iter().next()) {
        let email_tokens = if email.is_empty() {
            None
        } else {
            Some(quote! { .email(Some(#email)) })
        };
        quote! {
            utoipa::openapi::ContactBuilder::new()
                .name(Some(#name))
                #email_tokens
                .build()
        }
    } else {
        quote! {
            utoipa::openapi::Contact::default()
        }
    }
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
}
