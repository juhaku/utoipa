use proc_macro2::Ident;
use quote::{quote, format_ident};
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::token::{Comma, PathSep};
use syn::{Attribute, ExprPath, ItemMod, LitStr, PathSegment};

use crate::{parse_utils, ResultExt};

pub struct ModuleAttr {
    scope: String, // TODO should it also support expr??
}

impl Parse for ModuleAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        dbg!(&input);

        let name = &*input.parse::<Ident>()?.to_string();
        let scope = if name == "scope" {
            parse_utils::parse_next(input, || input.parse::<LitStr>())?.value()
        } else {
            String::new()
        };

        Ok(Self { scope })
    }
}

pub struct Module {}

impl Module {
    pub fn new(attributes: ModuleAttr, item_mod: ItemMod) -> Self {
        let name = item_mod.ident.to_string();
        item_mod.content.as_ref().map(|content| {
            content.1.iter().filter_map(|item| match item {
                syn::Item::Enum(item) if Self::is_utoipa_derive(&item.attrs, "ToSchema") => {
                    Some(ModuleItem::Schema(&item.ident))
                }
                syn::Item::Enum(item) if Self::is_utoipa_derive(&item.attrs, "ToResponse") => {
                    Some(ModuleItem::Response(&item.ident))
                }
                syn::Item::Struct(item) if Self::is_utoipa_derive(&item.attrs, "ToSchema") => {
                    Some(ModuleItem::Schema(&item.ident))
                }
                syn::Item::Struct(item) if Self::is_utoipa_derive(&item.attrs, "ToResponse") => {
                    Some(ModuleItem::Response(&item.ident))
                }
                syn::Item::Fn(item) if Self::is_utoipa_path(&item.attrs) => {
                    Some(ModuleItem::Path(&item.sig.ident))
                }
                _ => None,
            })
        });
        // TODO create schema for auto cfg module

        let autocfg_mod_path = format_ident!("{}::AutoCfgMod", name);
        let _ = quote! {
            pub struct AutoCfgMod;

            impl utoipa::OpenApi for AutoCfgMod {
                fn openapi() -> utoipa::openapi::OpenApi {
                    utoipa::openapi::OpenApiBuilder::new()
                        .components(
                            utoipa::openapi::schema::ComponentsBuilder::new()
                            .schema(name, schema)
                        )
                }
            }
        };

        Self {}
    }

    fn is_utoipa_derive(attributes: &[Attribute], derive: &str) -> bool {
        attributes
            .iter()
            .filter(|attribute| {
                attribute
                    .path()
                    .segments
                    .iter()
                    .any(|segment| segment.ident == "derive")
            })
            .flat_map(|derive| {
                derive
                    .parse_args_with(Punctuated::<ExprPath, Comma>::parse_terminated)
                    .unwrap_or_abort() // or return compile error???
            })
            .any(|derive_arg| {
                derive_arg
                    .path
                    .segments
                    .iter()
                    .any(|segment| segment.ident == derive)
            })
    }

    fn is_utoipa_path(attributes: &[Attribute]) -> bool {
        attributes.iter().any(|attribute| {
            attribute.path().segments.iter().all(|segment| {
                ["utoipa", "path"]
                    .iter()
                    .any(|expected| segment.ident == expected)
            })
        })
    }
}

pub(super) enum ModuleItem<'a> {
    Schema(&'a Ident),
    Response(&'a Ident),
    Path(&'a Ident),
}

// {
//     attrs: [],
//     vis: Visibility::Inherited,
//     unsafety: None,
//     mod_token: Mod,
//     ident: Ident {
//         ident: "innner_test",
//         span: #0 bytes(10060..10071),
//     },
//     content: Some(
//         (
//             Brace,
//             [
//                 Item::Struct {
//                     attrs: [
//                         Attribute {
//                             pound_token: Pound,
//                             style: AttrStyle::Outer,
//                             bracket_token: Bracket,
//                             meta: Meta::List {
//                                 path: Path {
//                                     leading_colon: None,
//                                     segments: [
//                                         PathSegment {
//                                             ident: Ident {
//                                                 ident: "derive",
//                                                 span: #0 bytes(10132..10138),
//                                             },
//                                             arguments: PathArguments::None,
//                                         },
//                                     ],
//                                 },
//                                 delimiter: MacroDelimiter::Paren(
//                                     Paren,
//                                 ),
//                                 tokens: TokenStream [
//                                     Ident {
//                                         ident: "utoipa",
//                                         span: #0 bytes(10139..10145),
//                                     },
//                                     Punct {
//                                         ch: ':',
//                                         spacing: Joint,
//                                         span: #0 bytes(10145..10146),
//                                     },
//                                     Punct {
//                                         ch: ':',
//                                         spacing: Alone,
//                                         span: #0 bytes(10146..10147),
//                                     },
//                                     Ident {
//                                         ident: "ToSchema",
//                                         span: #0 bytes(10147..10155),
//                                     },
//                                 ],
//                             },
//                         },
//                         Attribute {
//                             pound_token: Pound,
//                             style: AttrStyle::Outer,
//                             bracket_token: Bracket,
//                             meta: Meta::List {
//                                 path: Path {
//                                     leading_colon: None,
//                                     segments: [
//                                         PathSegment {
//                                             ident: Ident {
//                                                 ident: "allow",
//                                                 span: #0 bytes(10168..10173),
//                                             },
//                                             arguments: PathArguments::None,
//                                         },
//                                     ],
//                                 },
//                                 delimiter: MacroDelimiter::Paren(
//                                     Paren,
//                                 ),
//                                 tokens: TokenStream [
//                                     Ident {
//                                         ident: "unused",
//                                         span: #0 bytes(10174..10180),
//                                     },
//                                 ],
//                             },
//                         },
//                     ],
//                     vis: Visibility::Inherited,
//                     struct_token: Struct,
//                     ident: Ident {
//                         ident: "Value",
//                         span: #0 bytes(10198..10203),
//                     },
//                     generics: Generics {
//                         lt_token: None,
//                         params: [],
//                         gt_token: None,
//                         where_clause: None,
//                     },
//                     fields: Fields::Named {
//                         brace_token: Brace,
//                         named: [
//                             Field {
//                                 attrs: [],
//                                 vis: Visibility::Inherited,
//                                 mutability: FieldMutability::None,
//                                 ident: Some(
//                                     Ident {
//                                         ident: "value",
//                                         span: #0 bytes(10218..10223),
//                                     },
//                                 ),
//                                 colon_token: Some(
//                                     Colon,
//                                 ),
//                                 ty: Type::Path {
//                                     qself: None,
//                                     path: Path {
//                                         leading_colon: None,
//                                         segments: [
//                                             PathSegment {
//                                                 ident: Ident {
//                                                     ident: "String",
//                                                     span: #0 bytes(10225..10231),
//                                                 },
//                                                 arguments: PathArguments::None,
//                                             },
//                                         ],
//                                     },
//                                 },
//                             },
//                             Comma,
//                         ],
//                     },
//                     semi_token: None,
//                 },
//                 Item::Fn {
//                     attrs: [
//                         Attribute {
//                             pound_token: Pound,
//                             style: AttrStyle::Outer,
//                             bracket_token: Bracket,
//                             meta: Meta::List {
//                                 path: Path {
//                                     leading_colon: None,
//                                     segments: [
//                                         PathSegment {
//                                             ident: Ident {
//                                                 ident: "utoipa",
//                                                 span: #0 bytes(10254..10260),
//                                             },
//                                             arguments: PathArguments::None,
//                                         },
//                                         PathSep,
//                                         PathSegment {
//                                             ident: Ident {
//                                                 ident: "path",
//                                                 span: #0 bytes(10262..10266),
//                                             },
//                                             arguments: PathArguments::None,
//                                         },
//                                     ],
//                                 },
//                                 delimiter: MacroDelimiter::Paren(
//                                     Paren,
//                                 ),
//                                 tokens: TokenStream [
//                                     Ident {
//                                         ident: "get",
//                                         span: #0 bytes(10267..10270),
//                                     },
//                                     Punct {
//                                         ch: ',',
//                                         spacing: Alone,
//                                         span: #0 bytes(10270..10271),
//                                     },
//                                     Ident {
//                                         ident: "path",
//                                         span: #0 bytes(10272..10276),
//                                     },
//                                     Punct {
//                                         ch: '=',
//                                         spacing: Alone,
//                                         span: #0 bytes(10277..10278),
//                                     },
//                                     Literal {
//                                         kind: Str,
//                                         symbol: "/item",
//                                         suffix: None,
//                                         span: #0 bytes(10279..10286),
//                                     },
//                                 ],
//                             },
//                         },
//                     ],
//                     vis: Visibility::Inherited,
//                     sig: Signature {
//                         constness: None,
//                         asyncness: None,
//                         unsafety: None,
//                         abi: None,
//                         fn_token: Fn,
//                         ident: Ident {
//                             ident: "get_item",
//                             span: #0 bytes(10300..10308),
//                         },
//                         generics: Generics {
//                             lt_token: None,
//                             params: [],
//                             gt_token: None,
//                             where_clause: None,
//                         },
//                         paren_token: Paren,
//                         inputs: [],
//                         variadic: None,
//                         output: ReturnType::Default,
//                     },
//                     block: Block {
//                         brace_token: Brace,
//                         stmts: [],
//                     },
//                 },
//             ],
//         ),
//     ),
//     semi: None,
// }
