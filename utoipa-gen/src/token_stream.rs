use proc_macro2::{Span, TokenStream};
use quote::{quote_spanned, ToTokens};
use std::borrow::Borrow;
use std::borrow::Cow;
use std::error::Error;
use std::fmt::Display;

pub trait ToTokensDiagnostics {
    fn to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostics>;

    #[allow(unused)]
    fn into_token_stream(self) -> TokenStream
    where
        Self: std::marker::Sized,
    {
        ToTokensDiagnostics::to_token_stream(&self)
    }

    fn to_token_stream(&self) -> TokenStream {
        let mut tokens = TokenStream::new();
        match ToTokensDiagnostics::to_tokens(self, &mut tokens) {
            Ok(_) => tokens,
            Err(error_stream) => Into::<Diagnostics>::into(error_stream).into_token_stream(),
        }
    }

    fn try_to_token_stream(&self) -> Result<TokenStream, Diagnostics> {
        let mut tokens = TokenStream::new();
        match ToTokensDiagnostics::to_tokens(self, &mut tokens) {
            Ok(_) => Ok(tokens),
            Err(diagnostics) => Err(diagnostics),
        }
    }
}

impl<T: ToTokensDiagnostics> ToTokensDiagnostics for &'_ T {
    fn to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostics> {
        T::to_tokens(self, tokens)
    }
}

macro_rules! as_tokens_or_diagnostics {
    ( $type:expr ) => {{
        let mut _tokens = proc_macro2::TokenStream::new();
        match crate::token_stream::ToTokensDiagnostics::to_tokens($type, &mut _tokens) {
            Ok(_) => _tokens,
            Err(diagnostics) => return Err(diagnostics),
        }
    }};
}

pub(crate) use as_tokens_or_diagnostics;

/// A [`quote::quote!`] style macro that additionally supports interpolating types that
/// implement [`ToTokensDiagnostics`].
///
/// Works identically to [`quote!`][quote]` **except**:
/// - Returns [`Result<proc_macro2::TokenStream, Diagnostics>`] instead of[`proc_macro2::TokenStream`].
/// - Supports a `@ident` sigil for values whose type implements [`ToTokensDiagnostics`]. The  macro
///   calls [`ToTokensDiagnostics::to_tokens`] internally and propagates any [`Diagnostics`]  error
///   via early return.
/// - Regular `#ident` interpolation continues to work as in `quote!` for types that implement
///   [`quote::ToTokens`].
///
/// # Example
///
/// ```rust,ignore
/// # use quote::quote;
/// // Before — manual pre-conversion required
/// let request_body = as_tokens_or_diagnostics!(request_body);
/// let responses = as_tokens_or_diagnostics!(&responses);
/// tokens.extend(quote! {
///     .request_body(Some(#request_body))
///     .responses(#responses)
/// });
///
/// // After — inline via @sigil, ? propagates Diagnostics
/// tokens.extend(quote_diagnostics! {
///     .request_body(Some(@request_body))
///     .responses(@responses)
/// }?);
/// ```
/// [quote](https://docs.rs/quote/latest/quote/macro.quote.html).
macro_rules! quote_diagnostics {
    // flush
    (%qd $out:ident [$($prev:tt)*] []) => {
        $out.extend(quote::quote!($($prev)*));
    };
    // match @diagnostics
    (%qd $out:ident [$($prev:tt)*] [ @ $var:ident $($rest:tt)*] ) => {
        $out.extend(quote::quote!($($prev)*));
        let tokens = $crate::token_stream::as_tokens_or_diagnostics!(&$var);
        $out.extend(tokens);
        $crate::token_stream::quote_diagnostics!( %qd $out [ ] [ $($rest)* ]);
    };
    // inner ()
    (%qd $out:ident [$($prev:tt)*] [($($inner:tt)*) $($rest:tt)*]) => {
        $out.extend(quote::quote!($($prev)*));

        let mut group = proc_macro2::TokenStream::new();
        $crate::token_stream::quote_diagnostics!( %qd group [ ] [ $($inner)* ]);
        $out.extend(quote::quote!{ ( #group ) });

        $crate::token_stream::quote_diagnostics!( %qd $out [ ] [ $($rest)* ]);
    };
    // inner []
    (%qd $out:ident [$($prev:tt)*] [[ $($inner:tt)* ] $($rest:tt)*]) => {
        $out.extend(quote::quote!($($prev)*));

        let mut group = proc_macro2::TokenStream::new();
        $crate::token_stream::quote_diagnostics!( %qd group [ ] [ $($inner)* ]);
        $out.extend(quote::quote!{ [ #group ] });

        $crate::token_stream::quote_diagnostics!( %qd $out [ ] [ $($rest)* ]);
    };
    // inner {}
    (%qd $out:ident [$($prev:tt)*] [{ $($inner:tt)* } $($rest:tt)*]) => {
        $out.extend(quote::quote!($($prev)*));

        let mut group = proc_macro2::TokenStream::new();
        $crate::token_stream::quote_diagnostics!( %qd group [ ] [ $($inner)* ]);
        $out.extend(quote::quote!{ { #group } });

        $crate::token_stream::quote_diagnostics!( %qd $out [ ] [ $($rest)* ]);
    };
    (%qd $out:ident [$($prev:tt)*] [$first:tt $($rest:tt)*]) => {
        $crate::token_stream::quote_diagnostics!( %qd $out [ $($prev)* $first ] [ $($rest)* ]);
    };
    // begin
    ( $($tt:tt)* ) => {
        (|| -> Result<proc_macro2::TokenStream, crate::Diagnostics> {
            let mut tokens = proc_macro2::TokenStream::new();
            $crate::token_stream::quote_diagnostics!( %qd tokens [ ] [ $($tt)* ]);

            Ok(tokens)
        })()
    };
}

pub(crate) use quote_diagnostics;

/// A [`quote::quote_spanned!`] style macro that additionally supports interpolating types that
/// implement [`ToTokensDiagnostics`].
///
/// Behaves identically to [`quote_diagnostics`] but accepts a [`Span`] as the first argument
/// followed by `=>`, applying the span to the all emitted tokens. Returns
/// [`Result<proc_macro2::TokenStream, Diagnostics>`].
///
/// Use the `@ident` sigil to interpolate values whose type implements [`ToTokensDiagnostics`].
/// Regular `#ident` interpolation works as in [`quote::quote_spanned!`].
macro_rules! quote_diagnostics_spanned {
    // flush
    (%qd $out:ident $span:expr; [$($prev:tt)*] []) => {
        $out.extend(quote::quote_spanned!($span => $($prev)*));
    };
    // match @diagnostics
    (%qd $out:ident $span:expr; [$($prev:tt)*] [ @ $var:ident $($rest:tt)*] ) => {
        $out.extend(quote::quote_spanned!($span => $($prev)*));
        let tokens = $crate::token_stream::as_tokens_or_diagnostics!(&$var);
        $out.extend(tokens);
        $crate::token_stream::quote_diagnostics_spanned!( %qd $out $span; [ ] [ $($rest)* ]);
    };
    // inner ()
    (%qd $out:ident $span:expr; [$($prev:tt)*] [($($inner:tt)*) $($rest:tt)*]) => {
        $out.extend(quote::quote_spanned!($span => $($prev)*));

        let mut group = proc_macro2::TokenStream::new();
        $crate::token_stream::quote_diagnostics_spanned!( %qd group $span; [ ] [ $($inner)* ]);
        $out.extend(quote::quote_spanned!{ $span => ( #group ) });

        $crate::token_stream::quote_diagnostics_spanned!( %qd $out $span; [ ] [ $($rest)* ]);
    };
    // inner []
    (%qd $out:ident $span:expr; [$($prev:tt)*] [[ $($inner:tt)* ] $($rest:tt)*]) => {
        $out.extend(quote::quote_spanned!($span => $($prev)*));

        let mut group = proc_macro2::TokenStream::new();
        $crate::quote_diagnostics_spanned!( %qd group $span; [ ] [ $($inner)* ]);
        $out.extend(quote::quote_spanned!{$span =>  [ #group ] });

        $crate::quote_diagnostics_spanned!( %qd $out $span; [ ] [ $($rest)* ]);
    };
    // inner {}
    (%qd $out:ident $span:expr; [$($prev:tt)*] [{ $($inner:tt)* } $($rest:tt)*]) => {
        $out.extend(quote::quote_spanned!($span => $($prev)*));

        let mut group = proc_macro2::TokenStream::new();
        $crate::token_stream::quote_diagnostics_spanned!( %qd group $span; [ ] [ $($inner)* ]);
        $out.extend(quote::quote_spanned!{$span =>  { #group } });

        $crate::quote_diagnostics_spanned!( %qd $out $span; [ ] [ $($rest)* ]);
    };
    (%qd $out:ident $span:expr; [$($prev:tt)*] [$first:tt $($rest:tt)*]) => {
        $crate::token_stream::quote_diagnostics_spanned!( %qd $out $span; [ $($prev)* $first ] [ $($rest)* ]);
    };
    // begin
    ( $span:expr => $($tt:tt)* ) => {
        (|| -> Result<proc_macro2::TokenStream, crate::Diagnostics> {
            let mut tokens = proc_macro2::TokenStream::new();
            $crate::token_stream::quote_diagnostics_spanned!( %qd tokens $span; [ ] [ $($tt)* ]);

            Ok(tokens)
        })()
    };
}

pub(crate) use quote_diagnostics_spanned;

#[derive(Debug)]
pub struct Diagnostics {
    diagnostics: Vec<DiangosticsInner>,
}

#[derive(Debug)]
pub struct DiangosticsInner {
    span: Span,
    message: Cow<'static, str>,
    suggestions: Vec<Suggestion>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Suggestion {
    Help(Cow<'static, str>),
    Note(Cow<'static, str>),
}

impl Display for Diagnostics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl Display for Suggestion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Help(help) => {
                let s: &str = help.borrow();
                write!(f, "help = {}", s)
            }
            Self::Note(note) => {
                let s: &str = note.borrow();
                write!(f, "note = {}", s)
            }
        }
    }
}

impl Diagnostics {
    fn message(&self) -> Cow<'static, str> {
        self.diagnostics
            .first()
            .as_ref()
            .map(|diagnostics| diagnostics.message.clone())
            .unwrap_or_else(|| Cow::Borrowed(""))
    }

    pub fn new<S: Into<Cow<'static, str>>>(message: S) -> Self {
        Self::with_span(Span::call_site(), message)
    }

    pub fn with_span<S: Into<Cow<'static, str>>>(span: Span, message: S) -> Self {
        Self {
            diagnostics: vec![DiangosticsInner {
                span,
                message: message.into(),
                suggestions: Vec::new(),
            }],
        }
    }

    pub fn help<S: Into<Cow<'static, str>>>(mut self, help: S) -> Self {
        if let Some(diagnostics) = self.diagnostics.first_mut() {
            diagnostics.suggestions.push(Suggestion::Help(help.into()));
            diagnostics.suggestions.sort();
        }

        self
    }

    pub fn note<S: Into<Cow<'static, str>>>(mut self, note: S) -> Self {
        if let Some(diagnostics) = self.diagnostics.first_mut() {
            diagnostics.suggestions.push(Suggestion::Note(note.into()));
            diagnostics.suggestions.sort();
        }

        self
    }
}

impl From<syn::Error> for Diagnostics {
    fn from(value: syn::Error) -> Self {
        Self::with_span(value.span(), value.to_string())
    }
}

impl ToTokens for Diagnostics {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for diagnostics in &self.diagnostics {
            let span = diagnostics.span;
            let message: &str = diagnostics.message.borrow();

            let suggestions = diagnostics
                .suggestions
                .iter()
                .map(Suggestion::to_string)
                .collect::<Vec<_>>()
                .join("\n");

            let diagnostics = if !suggestions.is_empty() {
                Cow::Owned(format!("{message}\n\n{suggestions}"))
            } else {
                Cow::Borrowed(message)
            };

            tokens.extend(quote_spanned! {span=>
                ::core::compile_error!(#diagnostics);
            })
        }
    }
}

impl Error for Diagnostics {}

impl FromIterator<Diagnostics> for Option<Diagnostics> {
    fn from_iter<T: IntoIterator<Item = Diagnostics>>(iter: T) -> Self {
        iter.into_iter().reduce(|mut acc, diagnostics| {
            acc.diagnostics.extend(diagnostics.diagnostics);
            acc
        })
    }
}
