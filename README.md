# utoipa - Auto generated OpenAPI documentation

[![Utoipa build](https://github.com/juhaku/utoipa/actions/workflows/build.yaml/badge.svg)](https://github.com/juhaku/utoipa/actions/workflows/build.yaml)
[![crates.io](https://img.shields.io/static/v1?label=crates.io&message=0.1.1&color=orange&logo=rust)](https://crates.io/crates/utoipa/0.1.1)
[![docs.rs](https://img.shields.io/static/v1?label=docs.rs&message=utoipa&color=blue&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K)](https://docs.rs/utoipa/0.1.1/utoipa/)

Want to have your API documented with OpenAPI? But you dont want to see the
trouble with manual yaml or json tweaking? Would like it to be so easy that it would almost
be like utopic? Don't worry utoipa is just there to fill this gap. It aims to do if not all then
the most of heavy lifting for you enabling you to focus writing the actual API logic instead of
documentation. It aims to be *minimal*, *simple* and *fast*. It uses simple proc macros which
you can use to annotate your code to have items documented.

Utoipa crate provides auto generated OpenAPI documentation for Rust REST APIs. It treats
code first appoach as a first class citizen and simplifies API documentation by providing
simple macros for generating the documentation from your code.

It also contains Rust types of OpenAPI spec allowing you to write the OpenAPI spec only using
Rust if auto generation is not your flavor or does not fit your purpose.

Long term goal of the library is to be the place to go when OpenAPI documentation is needed in Rust
codebase.

Utoipa is framework agnostic and could be used together with any web framework or even without one. While 
being portable and standalone one of it's key aspects is simple integration with web frameworks. 

## Choose your flavor and document your API with ice cold IPA

Existing [examples](./examples) for following frameworks:

* **actix-web** 
* **warp**
* **tide**

**actix-web** has support support for actix specific parsing via **actix_extras** feature.

Even if there is no example for your favourite framework `utoipa` is portable and can be used with any 
web framework which supports decorating functions with macros similarly to **warp** and **tide** examples.

## What's up with the word play?

The name comes from words `utopic` and `api` where `uto` is the first three letters of _utopic_
and the `ipa` is _api_ reversed. Aaand... `ipa` is also awesome type of beer :beer:.

## Features

* **default** Default enabled features are **json**.
* **json** Enables **serde_json** what allow to use json values in OpenAPI specification values. Thus is
  enabled by default.
* **actix_extras** Enhances actix-web intgration with being able to parse some documentation
  from actix web macro attributes and types. See the [path attribute macro](https://docs.rs/utoipa/0.1.1/utoipa/attr.path.html) for more details.
* **debug** Add extra traits such as debug traits to openapi definitions and elsewhere.

## Install

Add minimal dependency declaration to Cargo.toml.
```
[dependencies]
utoipa = "0.1.1"
```

To enable more features such as use actix framework extras you could define the
dependency as follows.
```
[dependencies]
utoipa = { version = "0.1.1", features = ["actix_extras"] }
```

**Note!** To use `utoipa` together with Swagger UI you can use the [utoipa-swagger-ui](https://docs.rs/utoipa-swagger-ui/) crate.

## Examples

Create a struct or it could be an enum also. Add `Component` derive macro to it so it can be registered
as a component in OpenApi schema.
```rust
use utoipa::Component;

#[derive(Component)]
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
            (status = 404, description = "Pet was not found")
        ),
        params(
            ("id" = u64, path, description = "Pet database id to get Pet for"),
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

Tie the `Component` and the endpoint above to the OpenApi schema with following `OpenApi` derive proc macro.
```rust
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(handlers(pet_api::get_pet_by_id), components(Pet))]
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
      "email":"author email from Cargo.toml"
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
              "format": "int64"
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
        "required": [
          "id",
          "name"
        ],
        "properties": {
          "id": {
            "type": "integer",
            "format": "int64"
          },
          "name": {
            "type": "string"
          },
          "age": {
            "type": "integer",
            "format": "int32"
          }
        }
      }
    }
  }
}
```

## Go beyond the surface

* See how to serve OpenAPI doc via Swagger UI check [utoipa-swagger-ui](https://docs.rs/utoipa-swagger-ui/) crate for more details.
* Browse to [examples](https://github.com/juhaku/utoipa/tree/master/examples) for more comprehensinve examples.
* Modify generated OpenAPI at runtime check [Modify](https://docs.rs/utoipa/0.1.1/utoipa/trait.Modify.html) trait for more details.
* More about OpenAPI security in [security documentation](https://docs.rs/utoipa/0.1.1/utoipa/openapi/security/index.html).

# License

Licensed under either of [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) license at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate 
by you, shall be dual licensed, without any additional terms or conditions. 
