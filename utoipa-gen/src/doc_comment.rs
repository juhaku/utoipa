use syn::{Attribute, Expr, Lit, Meta};

const DOC_ATTRIBUTE_TYPE: &str = "doc";

/// CommentAttributes holds Vec of parsed doc comments
#[cfg_attr(feature = "debug", derive(Debug))]
pub(crate) struct CommentAttributes(pub(crate) Vec<String>);

impl CommentAttributes {
    /// Creates new [`CommentAttributes`] instance from [`Attribute`] slice filtering out all
    /// other attributes which are not `doc` comments
    pub(crate) fn from_attributes(attributes: &[Attribute]) -> Self {
        let mut docs = attributes
            .iter()
            .filter_map(|attr| {
                if !matches!(attr.path().get_ident(), Some(ident) if ident == DOC_ATTRIBUTE_TYPE) {
                    return None;
                }
                // ignore `#[doc(hidden)]` and similar tags.
                if let Meta::NameValue(name_value) = &attr.meta {
                    if let Expr::Lit(ref doc_comment) = name_value.value {
                        if let Lit::Str(ref doc) = doc_comment.lit {
                            let mut doc = doc.value();
                            // NB. Only trim trailing whitespaces. Leading whitespaces are handled
                            // below.
                            doc.truncate(doc.trim_end().len());
                            return Some(doc);
                        }
                    }
                }
                None
            })
            .collect::<Vec<_>>();
        // Calculate the minimum indentation of all non-empty lines and strip them.
        // This can get rid of typical single space after doc comment start `///`, but not messing
        // up indentation of markdown list or code.
        let min_indent = docs
            .iter()
            .filter(|s| !s.is_empty())
            // Only recognize ASCII space, not unicode multi-bytes ones.
            // `str::trim_ascii_start` requires 1.80 which is greater than our MSRV yet.
            .map(|s| s.len() - s.trim_start_matches(' ').len())
            .min()
            .unwrap_or(0);
        for line in &mut docs {
            if !line.is_empty() {
                line.drain(..min_indent);
            }
        }
        Self(docs)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns found `doc comments` as formatted `String` joining them all with `\n` _(new line)_.
    pub(crate) fn as_formatted_string(&self) -> String {
        self.0.join("\n")
    }
}
