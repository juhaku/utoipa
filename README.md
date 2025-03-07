# utoipa - Auto-generated OpenAPI documentation

[![Utoipa build](https://github.com/juhaku/utoipa/actions/workflows/build.yaml/badge.svg)](https://github.com/juhaku/utoipa/actions/workflows/build.yaml)
[![crates.io](https://img.shields.io/crates/v/utoipa.svg?label=crates.io&color=orange&logo=rust)](https://crates.io/crates/utoipa)
[![docs.rs](https://img.shields.io/static/v1?label=docs.rs&message=utoipa&color=blue&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K)](https://docs.rs/utoipa/latest/utoipa/)
![MSRV](https://img.shields.io/static/v1?label=MSRV&message=1.75&color=orange&logo=rust)

Pronounced **_/u:ˈtoʊ:i.pɑ/_** or **_/u:ˈtoʊˌaɪ.piˈeɪ/_** whatever works better for you.

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

|Flavor|Support|
|--|--|
|[actix-web](https://github.com/actix/actix-web)|Parse path, path parameters and query parameters, recognize request body and response body, [`utoipa-actix-web` bindings](./utoipa-actix-web/README.md). See more at [docs](https://docs.rs/utoipa/latest/utoipa/attr.path.html#actix_extras-feature-support-for-actix-web)|
|[axum](https://github.com/tokio-rs/axum)|Parse path and query parameters, recognize request body and response body, [`utoipa-axum` bindings](./utoipa-axum/README.md). See more at [docs](https://docs.rs/utoipa/latest/utoipa/attr.path.html#axum_extras-feature-support-for-axum)|
|[rocket](https://github.com/SergioBenitez/Rocket)| Parse path, path parameters and query parameters, recognize request body and response body. See more at [docs](https://docs.rs/utoipa/latest/utoipa/attr.path.html#rocket_extras-feature-support-for-rocket)|
|Others*| Plain `utoipa` without extra flavor. This gives you all the basic benefits listed below in **[Features](#features)** section but with little less automation.|

> Others* = For example [warp](https://github.com/seanmonstar/warp) but could be anything.

Refer to the existing [examples](./examples) to find out more.

## Features

* OpenAPI 3.1
* Pluggable, easy setup and integration with frameworks. 
* No bloat, enable what you need.
* Support for generic types
  * **Note!**<br>
    Tuples, arrays and slices cannot be used as generic arguments on types. Types implementing `ToSchema` manually should not have generic arguments, as
    they are not composeable and will result compile error.
* Automatic schema collection from usages recursively. 
  * Request body from either handler function arguments (if supported by framework) or from `request_body` attribute.
  * Response body from response `body` attribute or response `content` attribute.
* Various OpenAPI visualization tools supported out of the box.
* Rust type aliases via [`utoipa-config`](./utoipa-config/README.md).

## What's up with the word play?

The name comes from the words `utopic` and `api` where `uto` are the first three letters of _utopic_
and the `ipa` is _api_ reversed. Aaand... `ipa` is also an awesome type of beer :beer:.

## Crate Features

- **`macros`** Enable `utoipa-gen` macros. **This is enabled by default.**
- **`yaml`**: Enables **serde_norway** serialization of OpenAPI objects.
- **`actix_extras`**: Enhances [actix-web](https://github.com/actix/actix-web/) integration with being able to
  parse `path`, `path` and `query` parameters from actix web path attribute macros. See
  [docs](https://docs.rs/utoipa/latest/utoipa/attr.path.html#actix_extras-feature-support-for-actix-web) or [examples](./examples) for more details.
- **`rocket_extras`**: Enhances [rocket](https://github.com/SergioBenitez/Rocket) framework integration with being
  able to parse `path`, `path` and `query` parameters from rocket path attribute macros. See [docs](https://docs.rs/utoipa/latest/utoipa/attr.path.html#rocket_extras-feature-support-for-rocket)
  or [examples](./examples) for more details.
- **`axum_extras`**: Enhances [axum](https://github.com/tokio-rs/axum) framework integration allowing users to use `IntoParams` without
  defining the `parameter_in` attribute. See [docs](https://docs.rs/utoipa/latest/utoipa/attr.path.html#axum_extras-feature-support-for-axum)
  or [examples](./examples) for more details.
- **`debug`**: Add extra traits such as debug traits to openapi definitions and elsewhere.
- **`chrono`**: Add support for [chrono](https://crates.io/crates/chrono) `DateTime`, `Date`, `NaiveDate`, `NaiveDateTime`, `NaiveTime` and `Duration`
  types. By default these types are parsed to `string` types with additional `format` information.
  `format: date-time` for `DateTime` and `NaiveDateTime` and `format: date` for `Date` and `NaiveDate` according
  [RFC3339](https://www.rfc-editor.org/rfc/rfc3339#section-5.6) as `ISO-8601`. To
  override default `string` representation users have to use `value_type` attribute to override the type.
  See [docs](https://docs.rs/utoipa/latest/utoipa/derive.ToSchema.html) for more details.
- **`time`**: Add support for [time](https://crates.io/crates/time) `OffsetDateTime`, `PrimitiveDateTime`, `Date`, and `Duration` types.
  By default these types are parsed as `string`. `OffsetDateTime` and `PrimitiveDateTime` will use `date-time` format. `Date` will use
  `date` format and `Duration` will not have any format. To override default `string` representation users have to use `value_type` attribute
  to override the type. See [docs](https://docs.rs/utoipa/latest/utoipa/derive.ToSchema.html) for more details.
- **`jiff_0_2`** Add support for [jiff 0.2](https://crates.io/crates/jiff) `Zoned`, and `civil::Date` types.
  By default these types are parsed as `string`. `Zoned` will use `date-time` format. `civil::Date` will use
  `date` format. To override default `string` representation users have to use `value_type` attribute
  to override the type. See [docs](https://docs.rs/utoipa/latest/utoipa/derive.ToSchema.html) for more details.
- **`decimal`**: Add support for [rust_decimal](https://crates.io/crates/rust_decimal) `Decimal` type. **By default**
  it is interpreted as `String`. If you wish to change the format you need to override the type.
  See the `value_type` in [component derive docs](https://docs.rs/utoipa/latest/utoipa/derive.ToSchema.html).
- **`decimal_float`**: Add support for [rust_decimal](https://crates.io/crates/rust_decimal) `Decimal` type. **By default**
  it is interpreted as `Number`. This feature is mutually exclusive with **decimal** and allow to change the default type used in your
  documentation for `Decimal` much like `serde_with_float` feature exposed by rust_decimal.
- **`uuid`**: Add support for [uuid](https://github.com/uuid-rs/uuid). `Uuid` type will be presented as `String` with
  format `uuid` in OpenAPI spec.
- **`ulid`**: Add support for [ulid](https://github.com/dylanhart/ulid-rs). `Ulid` type will be presented as `String` with
  format `ulid` in OpenAPI spec.
- **`url`**: Add support for [url](https://github.com/servo/rust-url). `Url` type will be presented as `String` with
  format `uri` in OpenAPI spec.
- **`smallvec`**: Add support for [smallvec](https://crates.io/crates/smallvec). `SmallVec` will be treated as `Vec`.
- **`openapi_extensions`**: Adds traits and functions that provide extra convenience functions.
  See the [`request_body` docs](https://docs.rs/utoipa/latest/utoipa/openapi/request_body) for an example.
- **`repr`**: Add support for [repr_serde](https://github.com/dtolnay/serde-repr)'s `repr(u*)` and `repr(i*)` attributes to unit type enums for
  C-like enum representation. See [docs](https://docs.rs/utoipa/latest/utoipa/derive.ToSchema.html) for more details.
- **`preserve_order`**: Preserve order of properties when serializing the schema for a component.
  When enabled, the properties are listed in order of fields in the corresponding struct definition.
  When disabled, the properties are listed in alphabetical order.
- **`preserve_path_order`**: Preserve order of OpenAPI Paths according to order they have been
  introduced to the `#[openapi(paths(...))]` macro attribute. If disabled the paths will be
  ordered in alphabetical order. **However** the operations order under the path **will** be always constant according to [specification](https://spec.openapis.org/oas/latest.html#fixed-fields-6)
- **`indexmap`**: Add support for [indexmap](https://crates.io/crates/indexmap). When enabled `IndexMap` will be rendered as a map similar to
  `BTreeMap` and `HashMap`.
- **`non_strict_integers`**: Add support for non-standard integer formats `int8`, `int16`, `uint8`, `uint16`, `uint32`, and `uint64`.
- **`rc_schema`**: Add `ToSchema` support for `Arc<T>` and `Rc<T>` types. **Note!** serde `rc` feature flag must be enabled separately to allow
  serialization and deserialization of `Arc<T>` and `Rc<T>` types. See more about [serde feature flags](https://serde.rs/feature-flags.html).
- **`config`** Enables [`utoipa-config`](./utoipa-config/README.md) for the project which allows defining global configuration options for `utoipa`.

### Default Library Support

* Implicit partial support for `serde` attributes. See [docs](https://docs.rs/utoipa/latest/utoipa/derive.ToSchema.html#partial-serde-attributes-support) for more details.
* Support for [http](https://crates.io/crates/http) `StatusCode` in responses.

## Install

Add dependency declaration to `Cargo.toml`.

```toml
[dependencies]
utoipa = "5"
```

## Examples

_Create type with `ToSchema` and use it in `#[utoipa::path(...)]` that is registered to the `OpenApi`._

```rust
use utoipa::{OpenApi, ToSchema};

#[derive(ToSchema)]
struct Pet {
   id: u64,
   name: String,
   age: Option<i32>,
}

mod pet_api {
    /// Get pet by id
    ///
    /// Get pet from database by pet id
    #[utoipa::path(
        get,
        path = "/pets/{id}",
        responses(
            (status = 200, description = "Pet found successfully", body = Pet),
            (status = NOT_FOUND, description = "Pet was not found")
        ),
        params(
            ("id" = u64, Path, description = "Pet database id to get Pet for"),
        )
    )]
    async fn get_pet_by_id(pet_id: u64) -> Result<Pet, NotFound> {
        Ok(Pet {
            id: pet_id,
            age: None,
            name: "lightning".to_string(),
        })
    }
}

#[derive(OpenApi)]
#[openapi(paths(pet_api::get_pet_by_id))]
struct ApiDoc;

println!("{}", ApiDoc::openapi().to_pretty_json().unwrap());
```

<details>
    <summary><i><b>Above example will produce an OpenAPI doc like this:</b></i></summary>

```json
{
  "openapi": "3.1.0",
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
        "tags": [
          "pet_api"
        ],
        "summary": "Get pet by id",
        "description": "Get pet from database by pet id",
        "operationId": "get_pet_by_id",
        "parameters": [
          {
            "name": "id",
            "in": "path",
            "description": "Pet database id to get Pet for",
            "required": true,
            "schema": {
              "type": "integer",
              "format": "int64",
              "minimum": 0
            }
          }
        ],
        "responses": {
          "200": {
            "description": "Pet found successfully",
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
        }
      }
    }
  },
  "components": {
    "schemas": {
      "Pet": {
        "type": "object",
        "required": [
          "id",
          "name"
        ],
        "properties": {
          "age": {
            "type": [
              "integer",
              "null"
            ],
            "format": "int32"
          },
          "id": {
            "type": "integer",
            "format": "int64",
            "minimum": 0
          },
          "name": {
            "type": "string"
          }
        }
      }
    }
  }
}
```

</details>

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

## FAQ

### Swagger UI returns 404 NotFound from built binary

This is highly probably due to `RustEmbed` not embedding the Swagger UI to the executable. This is natural since the `RustEmbed`
library **does not** by default embed files on debug builds. To get around this you can do one of the following.

1. Build your executable in `--release` mode
2. or add `debug-embed` feature flag to your `Cargo.toml` for `utoipa-swagger-ui`. This will enable the `debug-emebed` feature flag for
   `RustEmbed` as well. Read more about this [here](https://github.com/juhaku/utoipa/issues/527#issuecomment-1474219098) and [here](https://github.com/juhaku/utoipa/issues/268).

Find `utoipa-swagger-ui` [feature flags here](https://github.com/juhaku/utoipa/tree/master/utoipa-swagger-ui#crate-features).

### How to implement `ToSchema` for external type?

There are few ways around this that are elaborated [here in detail](https://github.com/juhaku/utoipa/issues/790#issuecomment-1787754185).

### Auto discover for OpenAPI schemas and paths?

Currently there is no build in solution to automatically discover the OpenAPI types but for your luck there is a pretty neat crate that 
just does this for you called [utoipauto](https://github.com/ProbablyClem/utoipauto).

## License

Licensed under either of [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) license at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate
by you, shall be dual licensed, without any additional terms or conditions.
