use proc_macro2::{Ident, Span};
use proc_macro_error::{abort_call_site, emit_warning, OptionExt, ResultExt};
use syn::{Attribute, Lit, Meta};

const DOC_ATTRIBUTE_TYPE: &str = "doc";

/// CommentAttributes holds Vec of parsed doc comments
#[cfg_attr(feature = "debug", derive(Debug))]
pub(crate) struct CommentAttributes(pub(crate) Vec<String>);

impl CommentAttributes {
    /// Creates new [`CommentAttributes`] instance from [`Attribute`] slice filtering out all
    /// other attributes which are not `doc` comments
    pub(crate) fn from_attributes(attributes: &[Attribute]) -> Self {
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
            .expect_or_abort("Expected doc attribute with one path segment")
    }

    fn as_string_vec<'a, I: Iterator<Item = &'a Attribute>>(attributes: I) -> Vec<String> {
        attributes
            .into_iter()
            .filter_map(Self::parse_doc_comment)
            .collect()
    }

    fn parse_doc_comment(attribute: &Attribute) -> Option<String> {
        let meta = attribute.parse_meta().unwrap_or_abort();

        // TODO find a correct span?
        match meta {
            Meta::NameValue(name_value) => {
                if let Lit::Str(doc_comment) = name_value.lit {
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
