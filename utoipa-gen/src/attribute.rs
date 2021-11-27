use proc_macro2::{Ident, Span};
use proc_macro_error::{abort_call_site, emit_warning, OptionExt, ResultExt};
use syn::{Attribute, Meta};

const DOC_ATTRIBUTE_TYPE: &str = "doc";

pub struct CommentAttributes(pub Vec<String>);

impl CommentAttributes {
    pub fn from_attributes(attributes: &[Attribute]) -> Self {
        Self {
            0: Self::as_string_vec(attributes.iter().filter(Self::is_doc_attribute)),
        }
    }

    fn is_doc_attribute(attribute: &&Attribute) -> bool {
        &*Self::get_attribute_ident(attribute).to_string() == DOC_ATTRIBUTE_TYPE
    }

    fn get_attribute_ident(attribute: &Attribute) -> &Ident {
        attribute
            .path
            .get_ident()
            .expect_or_abort("Expected doc attribute")
    }

    fn as_string_vec<'a, I: Iterator<Item = &'a Attribute>>(attributes: I) -> Vec<String> {
        attributes
            .into_iter()
            .filter_map(Self::parse_doc_comment)
            .collect()
    }

    fn parse_doc_comment(attribute: &Attribute) -> Option<String> {
        let meta = attribute.parse_meta().unwrap_or_abort();

        match meta {
            Meta::NameValue(name_value) => {
                if let syn::Lit::Str(doc_comment) = name_value.lit {
                    Some(doc_comment.value().trim().to_string())
                } else {
                    emit_warning!(
                        Span::call_site(),
                        "Expected Lit::Str types for types in meta, ignoring value"
                    );
                    None
                }
            }
            _ => abort_call_site!("Exected only Meta::NameValue type"),
        }
    }
}
