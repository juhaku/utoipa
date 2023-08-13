# utoipa - Auto-generated OpenAPI documentation

[![Utoipa build](https://github.com/juhaku/utoipa/actions/workflows/build.yaml/badge.svg)](https://github.com/juhaku/utoipa/actions/workflows/build.yaml)
[![crates.io](https://img.shields.io/crates/v/utoipa.svg?label=crates.io&color=orange&logo=rust)](https://crates.io/crates/utoipa)
[![docs.rs](https://img.shields.io/static/v1?label=docs.rs&message=utoipa&color=blue&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K)](https://docs.rs/utoipa/latest/utoipa/)
![MSRV](https://img.shields.io/static/v1?label=MSRV&message=1.60%2B&color=orange&logo=rust)

Pronounced **_/ju:ˈtoʊ:i.pɑ/_** or **_/ju:ˈtoʊˌaɪ.piˈeɪ/_** whatever works better for you.

Want to have your API documented with OpenAPI? But don't want to be bothered
with manual YAML or JSON tweaking? Would like it to be so easy that it would almost
be utopic? Don't worry: utoipa is here to fill this gap. It aims to do, if not all, then
most of the heavy lifting for you, enabling you to focus on writing the actual API logic instead of
documentation. It aims to be _minimal_, _simple_ and _fast_. It uses simple `proc` macros which
you can use to annotate your code to have items documented.

The `utoipa` crate provides auto-generated OpenAPI documentation for Rust REST APIs. It treats
code-first approach as a first class citizen and simplifies API documentation by providing
simple macros for generating the documentation from your code.

It also contains Rust types of the OpenAPI spec, allowing you to write the OpenAPI spec only using
Rust if auto generation is not your flavor or does not fit your purpose.

Long term goal of the library is to be the place to go when OpenAPI documentation is needed in any Rust
codebase.

Utoipa is framework-agnostic, and could be used together with any web framework, or even without one. While
being portable and standalone, one of its key aspects is simple integration with web frameworks.

## Choose your flavor and document your API with ice-cold IPA

Refer to the existing [examples](./examples) for building the "todo" app in the following frameworks:

- **[actix-web](https://github.com/actix/actix-web)**
- **[axum](https://github.com/tokio-rs/axum)**
- **[warp](https://github.com/seanmonstar/warp)**
- **[tide](https://github.com/http-rs/tide)**
- **[rocket](https://github.com/SergioBenitez/Rocket)** (`0.4` and `0.5.0-rc3`)

All examples include a [Swagger-UI](https://github.com/swagger-api/swagger-ui) unless stated otherwise.

There are also examples of building multiple OpenAPI docs in one application, each separated in Swagger UI.
These examples exist only for the **actix** and **warp** frameworks.

Even if there is no example for your favourite framework, `utoipa` can be used with any
web framework which supports decorating functions with macros similarly to the **warp** and **tide** examples.

### Community examples

- **[graphul](https://github.com/graphul-rs/graphul/tree/main/examples/utoipa-swagger-ui)**
- **[salvo](https://github.com/salvo-rs/salvo/tree/main/examples/todos-openapi)**
- **[viz](https://github.com/viz-rs/viz/tree/main/examples/routing/openapi)**
- **[ntex](https://github.com/leon3s/ntex-rest-api-example)**

## What's up with the word play?

The name comes from the words `utopic` and `api` where `uto` are the first three letters of _utopic_
and the `ipa` is _api_ reversed. Aaand... `ipa` is also an awesome type of beer :beer:.

## Crate Features

- **yaml** Enables **serde_yaml** serialization of OpenAPI objects.
- **actix_extras** Enhances [actix-web](https://github.com/actix/actix-web/) integration with being able to
  parse `path`, `path` and `query` parameters from actix web path attribute macros. See
  [docs](https://docs.rs/utoipa/latest/utoipa/attr.path.html#actix_extras-support-for-actix-web) or [examples](./examples) for more details.
- **rocket_extras** Enhances [rocket](https://github.com/SergioBenitez/Rocket) framework integration with being
  able to parse `path`, `path` and `query` parameters from rocket path attribute macros. See [docs](https://docs.rs/utoipa/latest/utoipa/attr.path.html#rocket_extras-support-for-rocket)
  or [examples](./examples) for more details.
- **axum_extras** Enhances [axum](https://github.com/tokio-rs/axum) framework integration allowing users to use `IntoParams` without
  defining the `parameter_in` attribute. See [docs](https://docs.rs/utoipa/latest/utoipa/attr.path.html#axum_extras-feature-support-for-axum)
  or [examples](./examples) for more details.
- **debug** Add extra traits such as debug traits to openapi definitions and elsewhere.
- **chrono** Add support for [chrono](https://crates.io/crates/chrono) `DateTime`, `Date`, `NaiveDate`, `NaiveDateTime`, `NaiveTime` and `Duration`
  types. By default these types are parsed to `string` types with additional `format` information.
  `format: date-time` for `DateTime` and `NaiveDateTime` and `format: date` for `Date` and `NaiveDate` according
  [RFC3339](https://www.rfc-editor.org/rfc/rfc3339#section-5.6) as `ISO-8601`. To
  override default `string` representation users have to use `value_type` attribute to override the type.
  See [docs](https://docs.rs/utoipa/latest/utoipa/derive.ToSchema.html) for more details.
- **time** Add support for [time](https://crates.io/crates/time) `OffsetDateTime`, `PrimitiveDateTime`, `Date`, and `Duration` types.
  By default these types are parsed as `string`. `OffsetDateTime` and `PrimitiveDateTime` will use `date-time` format. `Date` will use
  `date` format and `Duration` will not have any format. To override default `string` representation users have to use `value_type` attribute
  to override the type. See [docs](https://docs.rs/utoipa/latest/utoipa/derive.ToSchema.html) for more details.
- **decimal** Add support for [rust_decimal](https://crates.io/crates/rust_decimal) `Decimal` type. **By default**
  it is interpreted as `String`. If you wish to change the format you need to override the type.
  See the `value_type` in [component derive docs](https://docs.rs/utoipa/latest/utoipa/derive.ToSchema.html).
- **uuid** Add support for [uuid](https://github.com/uuid-rs/uuid). `Uuid` type will be presented as `String` with
  format `uuid` in OpenAPI spec.
- **ulid** Add support for [ulid](https://github.com/dylanhart/ulid-rs). `Ulid` type will be presented as `String` with
  format `ulid` in OpenAPI spec.
- **smallvec** Add support for [smallvec](https://crates.io/crates/smallvec). `SmallVec` will be treated as `Vec`.
- **openapi_extensions** Adds traits and functions that provide extra convenience functions.
  See the [`request_body` docs](https://docs.rs/utoipa/latest/utoipa/openapi/request_body) for an example.
- **repr** Add support for [repr_serde](https://github.com/dtolnay/serde-repr)'s `repr(u*)` and `repr(i*)` attributes to unit type enums for
  C-like enum representation. See [docs](https://docs.rs/utoipa/latest/utoipa/derive.ToSchema.html) for more details.
- **preserve_order** Preserve order of properties when serializing the schema for a component.
  When enabled, the properties are listed in order of fields in the corresponding struct definition.
  When disabled, the properties are listed in alphabetical order.
- **preserve_path_order** Preserve order of OpenAPI Paths according to order they have been
  introduced to the `#[openapi(paths(...))]` macro attribute. If disabled the paths will be
  ordered in alphabetical order.
- **indexmap** Add support for [indexmap](https://crates.io/crates/indexmap). When enabled `IndexMap` will be rendered as a map similar to
  `BTreeMap` and `HashMap`.
- **non_strict_integers** Add support for non-standard integer formats `int8`, `int16`, `uint8`, `uint16`, `uint32`, and `uint64`.
- **rc_schema** Add `ToSchema` support for `Arc<T>` and `Rc<T>` types. **Note!** serde `rc` feature flag must be enabled separately to allow 
  serialization and deserialization of `Arc<T>` and `Rc<T>` types. See more about [serde feature flags](https://serde.rs/feature-flags.html).

Utoipa implicitly has partial support for `serde` attributes. See [docs](https://docs.rs/utoipa/latest/utoipa/derive.ToSchema.html#partial-serde-attributes-support) for more details.

## Install

Add minimal dependency declaration to Cargo.toml.

```toml
[dependencies]
utoipa = "3"
```

To enable more features such as use actix framework extras you could define the
dependency as follows.

```toml
[dependencies]
utoipa = { version = "3", features = ["actix_extras"] }
```

**Note!** To use `utoipa` together with Swagger UI you can use the [utoipa-swagger-ui](https://docs.rs/utoipa-swagger-ui/) crate.

## Examples

Create a struct, or it could also be an enum. Add `ToSchema` derive macro to it, so it can be registered
as an OpenAPI schema.

```rust
use utoipa::ToSchema;

#[derive(ToSchema)]
struct Pet {
   id: u64,
   name: String,
   age: Option<i32>,
}
```

Create a handler that would handle your business logic and add `path` proc attribute macro over it.

```rust
mod pet_api {
    /// Get pet by id
    ///
    /// Get pet from database by pet id
    #[utoipa::path(
        get,
        path = "/pets/{id}",
        responses(
            (status = 200, description = "Pet found succesfully", body = Pet),
            (status = NOT_FOUND, description = "Pet was not found")
        ),
        params(
            ("id" = u64, Path, description = "Pet database id to get Pet for"),
        )
    )]
    async fn get_pet_by_id(pet_id: u64) -> Pet {
        Pet {
            id: pet_id,
            age: None,
            name: "lightning".to_string(),
        }
    }
}
```
Utoipa has support for [http](https://crates.io/crates/http) `StatusCode` in responses.

_This attribute macro will create another struct named with `__path_` prefix + handler function name.
So when you implement `some_handler` function in different file and want to export this, make sure `__path_some_handler`
in the module can also be accessible from the root._

Tie the `Schema` and the endpoint above to the OpenAPI schema with following `OpenApi` derive proc macro.

```rust
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(pet_api::get_pet_by_id), components(schemas(Pet)))]
struct ApiDoc;

println!("{}", ApiDoc::openapi().to_pretty_json().unwrap());
```

This would produce api doc something similar to:

```json
{
  "openapi": "3.0.3",
  "info": {
    "title": "application name from Cargo.toml",
    "description": "description from Cargo.toml",
    "contact": {
      "name": "author name from Cargo.toml",
      "email": "author email from Cargo.toml"
    },
    "license": {
      "name": "license from Cargo.toml"
    },
    "version": "version from Cargo.toml"
  },
  "paths": {
    "/pets/{id}": {
      "get": {
        "tags": ["pet_api"],
        "summary": "Get pet by id",
        "description": "Get pet by id\n\nGet pet from database by pet id\n",
        "operationId": "get_pet_by_id",
        "parameters": [
          {
            "name": "id",
            "in": "path",
            "description": "Pet database id to get Pet for",
            "required": true,
            "deprecated": false,
            "schema": {
              "type": "integer",
              "format": "int64",
              "minimum": 0.0,
            }
          }
        ],
        "responses": {
          "200": {
            "description": "Pet found succesfully",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/Pet"
                }
              }
            }
          },
          "404": {
            "description": "Pet was not found"
          }
        },
        "deprecated": false
      }
    }
  },
  "components": {
    "schemas": {
      "Pet": {
        "type": "object",
        "required": ["id", "name"],
        "properties": {
          "id": {
            "type": "integer",
            "format": "int64",
            "minimum": 0.0,
          },
          "name": {
            "type": "string"
          },
          "age": {
            "type": "integer",
            "format": "int32",
            "nullable": true,
          }
        }
      }
    }
  }
}
```

 ## Modify OpenAPI at runtime

 You can modify generated OpenAPI at runtime either via generated types directly or using
 [Modify](https://docs.rs/utoipa/latest/utoipa/trait.Modify.html) trait.

 _Modify generated OpenAPI via types directly._
 ```rust
 #[derive(OpenApi)]
 #[openapi(
     info(description = "My Api description"),
 )]
 struct ApiDoc;

 let mut doc = ApiDoc::openapi();
 doc.info.title = String::from("My Api");
 ```

 _You can even convert the generated [OpenApi](https://docs.rs/utoipa/latest/utoipa/openapi/struct.OpenApi.html) to [OpenApiBuilder](https://docs.rs/utoipa/latest/utoipa/openapi/struct.OpenApiBuilder.html)._
 ```rust
 let builder: OpenApiBuilder = ApiDoc::openapi().into();
 ```

 See [Modify](https://docs.rs/utoipa/latest/utoipa/trait.Modify.html) trait for examples on how to modify generated OpenAPI via it.

## Go beyond the surface

- See how to serve OpenAPI doc via Swagger UI check [utoipa-swagger-ui](https://docs.rs/utoipa-swagger-ui/) crate for more details.
- Browse to [examples](https://github.com/juhaku/utoipa/tree/master/examples) for more comprehensive examples.
- Check [IntoResponses](https://docs.rs/utoipa/latest/utoipa/derive.IntoResponses.html) and [ToResponse](https://docs.rs/utoipa/latest/utoipa/derive.ToResponse.html) for examples on deriving responses.
- More about OpenAPI security in [security documentation](https://docs.rs/utoipa/latest/utoipa/openapi/security/index.html).
- Dump generated API doc to file at build time. See [issue 214 comment](https://github.com/juhaku/utoipa/issues/214#issuecomment-1179589373).

## General Pitfalls

### Swagger UI returns 404 NotFound from built binary

This is highly probably due to `RustEmbed` not embedding the Swagger UI to the executable. This is natural since the `RustEmbed`
library **does not** by default embed files on debug builds. To get around this you can do one of the following.

1. Build your executable in `--release` mode
2. or add `debug-embed` feature flag to your `Cargo.toml` for `utoipa-swagger-ui`. This will enable the `debug-emebed` feature flag for
   `RustEmbed` as well. Read more about this [here](https://github.com/juhaku/utoipa/issues/527#issuecomment-1474219098) and [here](https://github.com/juhaku/utoipa/issues/268).

Find `utoipa-swagger-ui` [feature flags here](https://github.com/juhaku/utoipa/tree/master/utoipa-swagger-ui#crate-features).

## License

Licensed under either of [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) license at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate
by you, shall be dual licensed, without any additional terms or conditions.
