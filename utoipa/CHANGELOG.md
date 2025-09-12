# Changelog - utoipa

**`utoipa`** is in direct correlation with **`utoipa-gen`** ([CHANGELOG.md](../utoipa-gen/CHANGELOG.md)). You might want
to look into changes introduced to **`utoipa-gen`**.

## 5.4.0 - Jun 16 2025

### Added

* Add support for jiff v0.2 (https://github.com/juhaku/utoipa/pull/1332)

### Changed

* Enhance ToSchema and ComposeSchema implementations for HashMap and HashSet to support custom hashers (https://github.com/juhaku/utoipa/pull/1319) 
* Replaced `serde_yaml` with `serde_norway` (https://github.com/juhaku/utoipa/pull/1311)

## 5.3.1 - Jan 6 2025

### Changed

* Update axum to v0.8 (https://github.com/juhaku/utoipa/pull/1269)
* Replace `assert-json-diff` with snapshot testing via `insta` (https://github.com/juhaku/utoipa/pull/1254)

## 5.3.0 - Dec 19 2024

### Fixed

* Fix diverging axum route and openapi spec (https://github.com/juhaku/utoipa/pull/1199)

### Changed

* Migrate to `utoipa-gen` `5.3.0` version (https://github.com/juhaku/utoipa/pull/1250)
* Use a re-exported `serde_json` dependency in macros instead of implicitly requiring it as dependency in end projects (https://github.com/juhaku/utoipa/pull/1243)

## 5.2.0 - Nov 2 2024

### Changed

* Incremented `utoipa-gen` version to `5.2.0`

## 5.1.3 - Oct 27 2024

### Changed 

* Updated `utoipa-gen` version

## 5.1.2 - Oct 16 2024

### Added

* Add implementation for utoipa-actix-web bindings (https://github.com/juhaku/utoipa/pull/1158)

### Changed

* Finalize actix-web utoipa bindings (https://github.com/juhaku/utoipa/pull/1160)

## 5.1.1 - Oct 16 2024

### Changed

*  Update utoipa-gen version

## 5.1.0 - Oct 16 2024

### Added

* Add `identifier` for `Info` (https://github.com/juhaku/utoipa/pull/1140)
* Implement schema traits for indexset (https://github.com/juhaku/utoipa/pull/1129)
* Add `ToSchema` implementation for serde_json::Value (https://github.com/juhaku/utoipa/pull/1132)

### Changed

* Change Option<T> compose to OneOfBuilder (https://github.com/juhaku/utoipa/pull/1141)

## 5.0.0 - Oct 14 2024

### Added

* Add missing `extensions` for OpenAPI types (https://github.com/juhaku/utoipa/pull/1104)
* Add a default impl of ToSchema::name() (https://github.com/juhaku/utoipa/pull/1096)
* Add support for `property_names` for object (https://github.com/juhaku/utoipa/pull/1084)
* Add changelogs for crates (https://github.com/juhaku/utoipa/pull/1075)
* Add explicit `Extensions` type (https://github.com/juhaku/utoipa/pull/1062)
* Add global config for `utiopa` (https://github.com/juhaku/utoipa/pull/1048)
* Add support for `links` in `#[utoipa::path]` (https://github.com/juhaku/utoipa/pull/1047)
* Add support for real generics (https://github.com/juhaku/utoipa/pull/1034)
* Add typos to CI (https://github.com/juhaku/utoipa/pull/1036)
* Add support for nullable schema map items (https://github.com/juhaku/utoipa/pull/1032)
* Add macros feature flag (https://github.com/juhaku/utoipa/pull/1015)
* Add extensions support for OpenApi (https://github.com/juhaku/utoipa/pull/1013)
* Add `utoipa-axum` binding example and update docs (https://github.com/juhaku/utoipa/pull/1007)
* Add support to define multiple operation methods (https://github.com/juhaku/utoipa/pull/1006)
* Add utoipa axum bindings (https://github.com/juhaku/utoipa/pull/1004)
* Add some deprecated attributes for `example` method
* Add nest `OpenApi` support (https://github.com/juhaku/utoipa/pull/930)
* Add `merge_from` method for chainable merge (https://github.com/juhaku/utoipa/pull/924)
* Add support for additional tags via `tags` (https://github.com/juhaku/utoipa/pull/916)

### Fixed 

* Fix impl `ToSchema` for container types (https://github.com/juhaku/utoipa/pull/1107)
* Fix typos in changelog
* Fix broken doc links
* Fix testing without explicit features (https://github.com/juhaku/utoipa/pull/1041)
* Fix negative value parsing on schema attributes (https://github.com/juhaku/utoipa/pull/1031)
* Fix default tag logic for paths (https://github.com/juhaku/utoipa/pull/1002)
* Fixed documentation spelling mistake (https://github.com/juhaku/utoipa/pull/999)
* Fix respect `required` attribute (https://github.com/juhaku/utoipa/pull/990)

### Changed

* Chore enhance generic schema collection (https://github.com/juhaku/utoipa/pull/1116)
* Enhance file uploads (https://github.com/juhaku/utoipa/pull/1113)
* Move `schemas` into `ToSchema` for schemas (https://github.com/juhaku/utoipa/pull/1112)
* List only `utoipa` related changes in `utoipa` CHANGELOG
* Remove commit commit id from changelogs (https://github.com/juhaku/utoipa/pull/1077)
* Update to rc
* Update README.md
* Chore change the operations implementation. (https://github.com/juhaku/utoipa/pull/1026)
* Make referenced schemas required (https://github.com/juhaku/utoipa/pull/1018)
* Enhance `utoipa-axum` bindings (https://github.com/juhaku/utoipa/pull/1017)
* Update next beta versions
* Chore update docs and relax `url` version (https://github.com/juhaku/utoipa/pull/1001)
* Bump up versions (https://github.com/juhaku/utoipa/pull/998)
* Add extensions for schemas (https://github.com/juhaku/utoipa/pull/983)
* Bump up to next alplha
* Update versions

### Breaking

* Adds support for `prefixItems` on `Array` (https://github.com/juhaku/utoipa/pull/1103)
* Implement automatic schema collection for requests (https://github.com/juhaku/utoipa/pull/1066)
* Refactor enums processing (https://github.com/juhaku/utoipa/pull/1059)
* Feature openapi 31 (https://github.com/juhaku/utoipa/pull/981)
* Enhance OpenApi nesting with tags support (https://github.com/juhaku/utoipa/pull/932)

## 4.2.3 - May 7 2024

### Changed

* Update utoipa version
* Make reqwest to use rustls instead of openssl (https://github.com/juhaku/utoipa/pull/912)
* Have OpenApi::merge consider operations/methods as well as paths (https://github.com/juhaku/utoipa/pull/910)

## 4.2.2 - May 7 2024

### Changed

* Fix utoipa-gen dependency version (https://github.com/juhaku/utoipa/pull/909)

## 4.2.1 - May 5 2024

### Added

* Add additional check to ensure generic type resolution is only for generics (https://github.com/juhaku/utoipa/pull/904)
* Add crate for serving Scalar via utoipa (https://github.com/juhaku/utoipa/pull/892)
* Add `default-features = false` to the optional axum dependency to avoid pulling in tokio in non-tokio environments (https://github.com/juhaku/utoipa/pull/874) 
* Add flex to `utoipa-swagger-ui` build (https://github.com/juhaku/utoipa/pull/845)

### Changed

* Seems like the zip_next is nowadays just zip again
* Update docs and next versions
* Use same licences for scalar as well
* Update default Swagger UI version (https://github.com/juhaku/utoipa/pull/905)
* Skip 1st line in path macro description expansion (https://github.com/juhaku/utoipa/pull/881)
* Implement include_str! for tags (https://github.com/juhaku/utoipa/pull/893)
* Replace `zip` with `zip_next` (https://github.com/juhaku/utoipa/pull/889)
* Refactor `ReDoc` to take `Cow<'static, str>` instead of borrowed `str` (https://github.com/juhaku/utoipa/pull/869)
* Refactor RapiDoc to take `Cow<'static, str>` instead of borrowed `str` (https://github.com/juhaku/utoipa/pull/867)
* Fix spelling (https://github.com/juhaku/utoipa/pull/846)

## 4.2.0 - Jan 9 2024

### Added

* Add support for specifying multiple security requirement keys (https://github.com/juhaku/utoipa/pull/813)

### Changed

* Update next versions
* Allowing utoipa/utoipa-swagger-ui successfully build on Windows and made path proc macro attribute more permissive (https://github.com/juhaku/utoipa/pull/830)
* Update Rocket v0.5 (https://github.com/juhaku/utoipa/pull/825)
* Generate embed code instead of using interpolation (https://github.com/juhaku/utoipa/pull/828)
* Update docs
* Path impl_for override. PathBuilder::path_from (https://github.com/juhaku/utoipa/pull/759)
* fix: fix typo (https://github.com/juhaku/utoipa/pull/822)
* Support serde deny_unknown_fields (https://github.com/juhaku/utoipa/pull/816)
* Misc document improvements (https://github.com/juhaku/utoipa/pull/814)
* Hide Debug behind debug feature (https://github.com/juhaku/utoipa/pull/815)
* Update next versions
* Axum 0.7 bindings (https://github.com/juhaku/utoipa/pull/807)

## 4.1.0 - Nov 13 2023

### Added

* feat: add HashSet and BTreeSet (https://github.com/juhaku/utoipa/pull/791)
* add openapi extensions "x-tokenName" (https://github.com/juhaku/utoipa/pull/763)

### Changed

* Update next versions
* Support `#[serde(flatten)]` for maps. (https://github.com/juhaku/utoipa/pull/799)
* Update utoipa versions

## 4.0.0 - Oct 7 2023

### Added

* Add Discriminator mapping (https://github.com/juhaku/utoipa/pull/752)
* Add test for date types in actix params (https://github.com/juhaku/utoipa/pull/758)
* Add `decimal_float` feature. (https://github.com/juhaku/utoipa/pull/750)

### Fixed

* Fix: panic on missing trailing / in rocket environment (https://github.com/juhaku/utoipa/pull/645) (#757)

### Changed

* Update next versions and dependencies
* enable required usage with schema_with attribute (https://github.com/juhaku/utoipa/pull/764)
* Allow additionalProperties to be an array (https://github.com/juhaku/utoipa/pull/756)
* Feat std::collections::LinkedList as a known field type for schema (https://github.com/juhaku/utoipa/pull/748)
* Feat url type (https://github.com/juhaku/utoipa/pull/747)

### Breaking

* Allow expression as macro arg (https://github.com/juhaku/utoipa/pull/762)

## 3.5.0 - Aug 20 2023

### Added

* Add support for serde skip in `IntoParams` derive (https://github.com/juhaku/utoipa/pull/743)
* Add rapidoc support (https://github.com/juhaku/utoipa/pull/723)
* Add redoc support for utoipa. (https://github.com/juhaku/utoipa/pull/720)

### Changed

* Update next versions
* Support ULID (https://github.com/juhaku/utoipa/pull/733)
* Update utoipa-swagger-ui version
* Update Swagger UI to 5.3.1
* Update README.md docs

## 3.4.4 - Aug 3 2023

### Added

* Add more axum path parameter tests
* Add descriptions to 2 variants of complex enums  (https://github.com/juhaku/utoipa/pull/714)
* Add support for #[schema(default = )] on user-defined types (https://github.com/juhaku/utoipa/pull/712) (#713)
* Adding "AnyOf" branch for Schema (https://github.com/juhaku/utoipa/pull/706)

### Fixed

* Fix generics actix example (https://github.com/juhaku/utoipa/pull/716)
* Fix typos in doc (https://github.com/juhaku/utoipa/pull/709)

### Changed

* Update next versions
* allow and ignore #[doc(...)] tags in ToSchema derive (https://github.com/juhaku/utoipa/pull/708)
* Allow setting titles on all OpenApi Schema types and allow descriptions to propagate for UnnamedStructSchema (https://github.com/juhaku/utoipa/pull/694)

## 3.4.3 - Jul 23 2023

### Fixed

* Fix automatic request body (https://github.com/juhaku/utoipa/pull/701)

### Changed

* Update next versions

## 3.4.2 - Jul 23 2023

### Fixed

* Fix `Arc<T>` and `Rc<T>` and `SmallVec<[T]>` (https://github.com/juhaku/utoipa/pull/699)
* Fix broken link and enforce workspace resolver

## 3.4.1 - Jul 22 2023

### Added

* Add support for deprecation using schema attribute (https://github.com/juhaku/utoipa/pull/688)
* Add enum path param test (https://github.com/juhaku/utoipa/pull/680)
* Add tests for uuid path params (https://github.com/juhaku/utoipa/pull/676)

### Fixed

* Fix `Option<Query<T>>` type support (https://github.com/juhaku/utoipa/pull/678)

### Changed 

* Update next versions
* Disable automatic parameter recognition (https://github.com/juhaku/utoipa/pull/696)
* Added support for Arc fields to be treated like Box or RefCell (https://github.com/juhaku/utoipa/pull/690)

## 3.4.0 - Jul 13 2023

### Added

* Add automatic body recognition for rocket (https://github.com/juhaku/utoipa/pull/670)
* Add automatic type recognition for axum (https://github.com/juhaku/utoipa/pull/668)
* Add automatic query parameter recognition (https://github.com/juhaku/utoipa/pull/666)
* Add support for chrono::NaiveTime (https://github.com/juhaku/utoipa/pull/641)
* Add `preserve_path_order` cargo feature docs (https://github.com/juhaku/utoipa/pull/614)
* Add automatic request body recognition (https://github.com/juhaku/utoipa/pull/589)
* Add docs and tests for aliases (https://github.com/juhaku/utoipa/pull/587)
* Add basic auto response type support (https://github.com/juhaku/utoipa/pull/582)
* Add preserve_path_order feature (https://github.com/juhaku/utoipa/pull/612)

### Fixed

* Fix utoipa-gen feature and update versions
* Fix broken links (https://github.com/juhaku/utoipa/pull/669)
* Fix tests for feature non_strict_integers (https://github.com/juhaku/utoipa/pull/619)

### Changed

* Update next versions
* Rename `auto_types` feature flag (https://github.com/juhaku/utoipa/pull/665)
* Chore add more feature flag checks for auto types
* Remove `type: object` restriction in empty() (https://github.com/juhaku/utoipa/pull/648)
* exclude const generic arguments from generic_types (https://github.com/juhaku/utoipa/pull/627)
* Make sure to parse a comma token after the status in IntoResponses (https://github.com/juhaku/utoipa/pull/630)
* Omit decimal zeros when serializing minimum/maximum/multiple (https://github.com/juhaku/utoipa/pull/618)
* Correct `with_schema` to `schema_with` in docs (https://github.com/juhaku/utoipa/pull/586)
* Improve docs for examples (https://github.com/juhaku/utoipa/pull/584)
* Use swagger-ui v4.18.2 (https://github.com/juhaku/utoipa/pull/585)

## 3.3.0 - Apr 16 2023

### Added

* Add more known formats (https://github.com/juhaku/utoipa/pull/571)
* Add `indexmap` feature support for `TypeTree`

### Fixed

* Fix Schema as additional properties (https://github.com/juhaku/utoipa/pull/580)
* Fix `preserve_order` feature (https://github.com/juhaku/utoipa/pull/562)

### Changed

* Update next release versions
* feat: Support deserializing other versions in 3.0.x (https://github.com/juhaku/utoipa/pull/578)
* Allow additional integer types (https://github.com/juhaku/utoipa/pull/575)
* Bump rocket to v0.5.0-rc.3 (https://github.com/juhaku/utoipa/pull/577)
* feat: Allow default value on Content::examples (https://github.com/juhaku/utoipa/pull/579)
* Allow value_type serde_json::Value (https://github.com/juhaku/utoipa/pull/568)
* Rename AdditionalProperties (https://github.com/juhaku/utoipa/pull/564)
* Cargo format
* Update utoipa-swagger-ui version

## 3.2.1 - Mar 31 2023

### Changed

* Update next release versions (https://github.com/juhaku/utoipa/pull/555)
* Rename /api-doc/ to /api-docs
* Don't rely on listed serde_json crate


## 3.2.0 - Mar 28 2023

### Added

* Add support for partial schema (https://github.com/juhaku/utoipa/pull/544)
* Add tuple support for component schema (https://github.com/juhaku/utoipa/pull/541)
* Add missing enum variant examples (https://github.com/juhaku/utoipa/pull/538)
* Add support for auto-populating field default values (https://github.com/juhaku/utoipa/pull/533)

### Fixed

* Fix untagged enum unit variant support (https://github.com/juhaku/utoipa/pull/545)
* bugfix: use `map()` instead of `unwrap()` (https://github.com/juhaku/utoipa/pull/536)

### Changed

* Update next release versions
* Refactor aliases support on `ToSchema` derive (https://github.com/juhaku/utoipa/pull/546)
* Upgrade to syn2 (https://github.com/juhaku/utoipa/pull/542)

## 3.1.2 - Mar 20 2023

### Added

* Add support for double number format (https://github.com/juhaku/utoipa/pull/526)

### Changed

* Update next versions
* Make `Option` non-required & add `required` attr (https://github.com/juhaku/utoipa/pull/530)
* Remove needles ToTokens import
* Clean up & clippy lint
* Unify component schema tokenization (https://github.com/juhaku/utoipa/pull/525)

## 3.1.1 - Mar 16 2023

### Added

* Add missing `As` attribute to complex enum (https://github.com/juhaku/utoipa/pull/516)
* Add support for chrono NaiveDateTime (https://github.com/juhaku/utoipa/pull/514)

### Fixed

* Fix empty contact creation (https://github.com/juhaku/utoipa/pull/517)

### Changed

* Update next versions
* Remove superfluous `deprecated` path attribute (https://github.com/juhaku/utoipa/pull/520)
* Cargo format
* Make unsigned integers implicityly minimum zero (https://github.com/juhaku/utoipa/pull/515)

## 3.1.0 - Mar 10 2023

### Added

* Add support for external OpenAPI docs (https://github.com/juhaku/utoipa/pull/502)
* Add full support for nullable field detection (https://github.com/juhaku/utoipa/pull/498)
* Add support for free form additional properties (https://github.com/juhaku/utoipa/pull/495)

### Fixed

* Fix nullable ref schema inline (https://github.com/juhaku/utoipa/pull/510)
* Fix nullable ref schema (https://github.com/juhaku/utoipa/pull/509)

### Changed

* Update next versions

## 3.0.3 - Feb 19 2023

### Added

* Add description support for object field. (https://github.com/juhaku/utoipa/pull/492)

### Fixed

* Fix clippy lint
* Fix function argument support for `#[utoipa::path]` (https://github.com/juhaku/utoipa/pull/489)
* Fix parsed version in info being ignored (https://github.com/juhaku/utoipa/pull/485)

### Changed

* Update next versions
* Improve description support on `ToSchema` fields (https://github.com/juhaku/utoipa/pull/490)
* Update OpenApi derive docs and tests

## 3.0.2 - Feb 10 2023

### Added

* Add support for unit type `()` (https://github.com/juhaku/utoipa/pull/464)

### Changed

* Update next versions
* Enhance unit type support (https://github.com/juhaku/utoipa/pull/476)
* Support arbitrary exprs in operation_id (https://github.com/juhaku/utoipa/pull/472)

## 3.0.1 - Jan 29 2023

### Fixed

* Fix explicit lifetimes for consts (https://github.com/juhaku/utoipa/pull/467)

### Changed

* Update next versions

## 3.0.0 - Jan 26 2023

Migration guide: https://github.com/juhaku/utoipa/discussions/456

### Added
* Add support for serde `skip_serializing` (https://github.com/juhaku/utoipa/pull/438)
* Add `preserve_order` feature to preserve property order during serialization (https://github.com/juhaku/utoipa/pull/436)
* Add derive `IntoResponses` support (https://github.com/juhaku/utoipa/pull/433)
* Add `ToResponse` derive implementation (https://github.com/juhaku/utoipa/pull/416)
* Add support for repeated `schema(...)` definition (https://github.com/juhaku/utoipa/pull/410)
* Add external ref(...) attribute (https://github.com/juhaku/utoipa/pull/409)
* Add example attributes for request body (https://github.com/juhaku/utoipa/pull/406)
* Add auto detect application/octet-stream type (https://github.com/juhaku/utoipa/pull/405)
* Add support for chrono NaiveDate (https://github.com/juhaku/utoipa/pull/404)
* Add support for multiple examples in response (https://github.com/juhaku/utoipa/pull/403)
* Add Example type to OpenApi types (https://github.com/juhaku/utoipa/pull/402)
* Add derive info support for derive OpenApi (https://github.com/juhaku/utoipa/pull/400)
* Add `merge` functionality for `OpenApi` (https://github.com/juhaku/utoipa/pull/397)
* Add derive servers attribute for OpenApi (https://github.com/juhaku/utoipa/pull/395)
* Add support for unit sructs (https://github.com/juhaku/utoipa/pull/392)
* Add support for `schema_with` custom fn reference (https://github.com/juhaku/utoipa/pull/390)
* Add support for multiple serde definitions (https://github.com/juhaku/utoipa/pull/389)
* Add support for tuple Path parameters for axum (https://github.com/juhaku/utoipa/pull/388)
* Add derive validation attributes to `IntoParams` (https://github.com/juhaku/utoipa/pull/386)
* Add support for derive validation attributes (https://github.com/juhaku/utoipa/pull/385)
* Add support for missing attributes for validation (https://github.com/juhaku/utoipa/pull/379)@juhaku 
* Add support for multiple return types (https://github.com/juhaku/utoipa/pull/377)
* Add support for self referencing schema (https://github.com/juhaku/utoipa/pull/375)
* Add missing features to `IntoParams` (https://github.com/juhaku/utoipa/pull/374)

### Fixed

* Fix spelling (https://github.com/juhaku/utoipa/pull/450)
* Fix empty string path parameter name for Axum (https://github.com/juhaku/utoipa/pull/424)
* Fix typo in doc
* Fix make untagged enum object variants required (https://github.com/juhaku/utoipa/pull/407)
* Fix time-crate typo in schema format tokens (https://github.com/juhaku/utoipa/pull/401)
* Fix primitive type generic aliases (https://github.com/juhaku/utoipa/pull/393)
* Fix `TypeTree` for `slice` and `array` types (https://github.com/juhaku/utoipa/pull/387)
* Fix minimum axum version info
* Fix broken swagger-ui on axum  (https://github.com/juhaku/utoipa/pull/370)

### Changed

* Fix typos in changelog
* Refactor `ToResponse` trait (https://github.com/juhaku/utoipa/pull/460)
* Refactor to schema casting as (https://github.com/juhaku/utoipa/pull/459)
* Enhance generic aliases with lifetimes support (https://github.com/juhaku/utoipa/pull/458)
* Enhance path tuple argument support (https://github.com/juhaku/utoipa/pull/455)
* Update versions
* Update README
* Update docs
* Improve docs (https://github.com/juhaku/utoipa/pull/444)
* Enhance responses derive support (https://github.com/juhaku/utoipa/pull/443)
* Document `preserve_order` feature flag (https://github.com/juhaku/utoipa/pull/437)
* Feat/serde enum representation (https://github.com/juhaku/utoipa/pull/414)
* Enhance `ToResponse` implementation (https://github.com/juhaku/utoipa/pull/419)
* Swagger UI url config (https://github.com/juhaku/utoipa/pull/418)
* Address clippy lint
* Improve documentation
* Enhance repeated attributes support (https://github.com/juhaku/utoipa/pull/411)
* Make derive OpenApi server variable names LitStr
* Refactor `Type` to `TypeTree` (https://github.com/juhaku/utoipa/pull/408)
* Update `ToSchema` documentation
* feat: make schema_type and title pub on Object (https://github.com/juhaku/utoipa/pull/382)
* Use BTreeMap for responses of components to make it fixed order (https://github.com/juhaku/utoipa/pull/380)
* Chore make `serde_json` mandatory dependency (https://github.com/juhaku/utoipa/pull/378)
* Feature http status codes (https://github.com/juhaku/utoipa/pull/376)
* Refactor some path derive with `IntoParmas` tests
* Update utoipa-swagger-ui install example
* Chore refine description attribute (https://github.com/juhaku/utoipa/pull/373)
* Update swagger-ui dependencies versions
* Update utoipa-swagger-ui version
* cargo format
* Update to axum 0.6.0 (https://github.com/juhaku/utoipa/pull/369)

