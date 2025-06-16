# Changelog - utoipa-gen

## 5.4.0 - Jun 16 2025

### Added

* Add feature support extensions in `utoipa::path` macro (https://github.com/juhaku/utoipa/pull/1292)
* Add support for jiff v0.2 (https://github.com/juhaku/utoipa/pull/1332)

## 5.3.1 - Jan 6 2025

### Fixed

* Fix bug in generic schemas on OpenApi derive macro (https://github.com/juhaku/utoipa/pull/1277)

### Changed

* Update axum to v0.8 (https://github.com/juhaku/utoipa/pull/1269)
* Replace `assert-json-diff` with snapshot testing via `insta` (https://github.com/juhaku/utoipa/pull/1253)
* scripts/test.sh: Fix `auto_into_responses` feature declaration (https://github.com/juhaku/utoipa/pull/1252)

## 5.3.0 - Dec 19 2024

### Fixed

* Fix tagged enum with flatten fields (https://github.com/juhaku/utoipa/pull/1208)

### Added

* Add `encoding` support for `request_body` (https://github.com/juhaku/utoipa/pull/1237)
* Add support for `#[schema(pattern = "...")]` on new type structs (https://github.com/juhaku/utoipa/pull/1241)

### Changed

* Adjust params code to not set `nullable` on `Option` for `Query` params (https://github.com/juhaku/utoipa/pull/1248)
* Use `insta` for snapshot testing (https://github.com/juhaku/utoipa/pull/1247)
* Make `parse_named_attributes` a method of `MediaTypeAttr` (https://github.com/juhaku/utoipa/pull/1236)
* Use a re-exported `serde_json` dependency in macros instead of implicitly requiring it as dependency in end projects (https://github.com/juhaku/utoipa/pull/1243)
* Simplified `ToTokensDiagnostics` for `request_body` (https://github.com/juhaku/utoipa/pull/1235)
* `Info::from_env()` sets `License::identifier` (https://github.com/juhaku/utoipa/pull/1233)

## 5.2.0 - Nov 2 2024

### Fixed

* Fix alias support for `IntoParams` (https://github.com/juhaku/utoipa/pull/1179)

### Changed

* Added missing formats for `KnownFormat` parsing (https://github.com/juhaku/utoipa/pull/1178)
* The `#[schema(ignore)]` attribute now accepts an optional bool value/function path (https://github.com/juhaku/utoipa/pull/1177)

## 5.1.3 - Oct 27 2024

### Fixed

* Fix `no_recursion` support on maps (https://github.com/juhaku/utoipa/pull/1167)

## 5.1.2 - Oct 16 2024

### Added

* Add implementation for utoipa-actix-web bindings (https://github.com/juhaku/utoipa/pull/1158)

### Changed

* Finalize actix-web utoipa bindings (https://github.com/juhaku/utoipa/pull/1160)

## 5.1.1 - Oct 16 2024

### Changed

*  Enhance no_recursion rule to apply also containers (https://github.com/juhaku/utoipa/pull/1144)

## 5.1.0 - Oct 16 2024

### Added

* Add `identifier` for `Info` (https://github.com/juhaku/utoipa/pull/1140)
* Add `no_recursion` feature for `ToSchema` (https://github.com/juhaku/utoipa/pull/1137)

### Fixed

* Chore explicit FromIterator for edition 2018 (https://github.com/juhaku/utoipa/pull/1131)

### Changed

- Switch to `oneOf` instead `allOf` to handle nullable values (https://github.com/juhaku/utoipa/pull/1135)

## 5.0.0 - Oct 14 2024

### Added

* Add support for title and description for unit struct schema (https://github.com/juhaku/utoipa/pull/1122)
* Add support for `schema(ignore)` and `param(ignore)` (https://github.com/juhaku/utoipa/pull/1090)
* Add support for `property_names` for object (https://github.com/juhaku/utoipa/pull/1084)
* Add `bound` attribute for customizing generic impl bounds. (https://github.com/juhaku/utoipa/pull/1079)
* Add auto collect schemas for utoipa-axum (https://github.com/juhaku/utoipa/pull/1072)
* Add global config for `utiopa` (https://github.com/juhaku/utoipa/pull/1048)
* Add support for `links` in `#[utoipa::path]` (https://github.com/juhaku/utoipa/pull/1047)
* Add support for `termsOfService` to OpenApi derive (https://github.com/juhaku/utoipa/pull/1046)
* Add typos to CI (https://github.com/juhaku/utoipa/pull/1036)
* Add test for logical or security requirement
* Add macros feature flag (https://github.com/juhaku/utoipa/pull/1015)
* Add parsing support for non strict integers (https://github.com/juhaku/utoipa/pull/1012)
* Add `utoipa-axum` binding example and update docs (https://github.com/juhaku/utoipa/pull/1007)
* Add utoipa axum bindings (https://github.com/juhaku/utoipa/pull/1004)
* Add some deprecated attributes for `example` method
* Add support for inlined enum variants (https://github.com/juhaku/utoipa/pull/963)
* Add `description` attribute on `ToSchema` (https://github.com/juhaku/utoipa/pull/949)
* Add support for description and summary overriding (https://github.com/juhaku/utoipa/pull/948)
* Add nest `OpenApi` support (https://github.com/juhaku/utoipa/pull/930)
* Add support for additional tags via `tags` (https://github.com/juhaku/utoipa/pull/916)

### Fixed

* Fix path rewrite (https://github.com/juhaku/utoipa/pull/1120)
* Chore filter const generics (https://github.com/juhaku/utoipa/pull/1118)
* Fix impl `ToSchema` for container types (https://github.com/juhaku/utoipa/pull/1107)
* Fix description on `inline` field (https://github.com/juhaku/utoipa/pull/1102)
* Fix `title` on unnamed struct and references (https://github.com/juhaku/utoipa/pull/1101)
* Fix generic references (https://github.com/juhaku/utoipa/pull/1097)
* Fix non generic root generic references (https://github.com/juhaku/utoipa/pull/1095)
* Fix option wrapped tailing query parameters (https://github.com/juhaku/utoipa/pull/1089)
* Fix doc comment trimming to keep relative indentation. (https://github.com/juhaku/utoipa/pull/1082)
* Fix generic aliases (https://github.com/juhaku/utoipa/pull/1083)
* Fix nest path config struct name (https://github.com/juhaku/utoipa/pull/1081)
* Fix `as` attribute path format (https://github.com/juhaku/utoipa/pull/1080)
* Fix allow response `content_type` without schema (https://github.com/juhaku/utoipa/pull/1073)
* Fix negative value parsing on schema attributes (https://github.com/juhaku/utoipa/pull/1031)
* Fix parameter inline for tuple path params (https://github.com/juhaku/utoipa/pull/1014)
* Fix some typos
* Fix default tag logic for paths (https://github.com/juhaku/utoipa/pull/1002)
* Fix respect `required` attribute (https://github.com/juhaku/utoipa/pull/990)
* Fix find actual request body TypeTree (https://github.com/juhaku/utoipa/pull/977)
* Fix summary / description split on empty lines (https://github.com/juhaku/utoipa/pull/947)
* Fix compile error propagation (https://github.com/juhaku/utoipa/pull/929)
* Fix tuple params missing features (https://github.com/juhaku/utoipa/pull/928)
* fix(utoipa-gen): remove unnecessary allocation with to_string in expanded code (https://github.com/juhaku/utoipa/pull/982)

### Changed

* Chore enhance generic schema collection (https://github.com/juhaku/utoipa/pull/1116)
* Enhance file uploads (https://github.com/juhaku/utoipa/pull/1113)
* Move `schemas` into `ToSchema` for schemas (https://github.com/juhaku/utoipa/pull/1112)
* Refactor `KnownFormat`
* Add path rewrite support (https://github.com/juhaku/utoipa/pull/1110)
* Fix broken tests
* Fix typos
* Update `utoipa-config` version
* Remove commit commit id from changelogs (https://github.com/juhaku/utoipa/pull/1077)
* Update to rc version
* Chore unify request body and ext request body (https://github.com/juhaku/utoipa/pull/1067)
* Refactor structs processing (https://github.com/juhaku/utoipa/pull/1060)
* Disable unused default features of rust_decimal (https://github.com/juhaku/utoipa/pull/1029)
* Make referenced schemas required (https://github.com/juhaku/utoipa/pull/1018)
* Update next beta versions
* Chore refactor `OpenApi` derive macro (https://github.com/juhaku/utoipa/pull/1011)
* Chore refactor Name trait usage in features (https://github.com/juhaku/utoipa/pull/1009)
* Chore refactor features (https://github.com/juhaku/utoipa/pull/1008)
* Chore update docs and relax `url` version (https://github.com/juhaku/utoipa/pull/1001)
* Bump up versions (https://github.com/juhaku/utoipa/pull/998)
* Clean up some unused fields
* Bump up to next alplha
* Update versions
* Address clippy lints and refactor serde parsing (https://github.com/juhaku/utoipa/pull/931)
* Clean up imports for utoipa-gen
* change pub(super) enum to pub enum (https://github.com/juhaku/utoipa/pull/926)
* Migrate out from proc macro error (https://github.com/juhaku/utoipa/pull/920)

### Breaking

* Adds support for `prefixItems` on `Array` (https://github.com/juhaku/utoipa/pull/1103)
* Auto collect tuple responses schema references (https://github.com/juhaku/utoipa/pull/1071)
* Implement automatic schema collection for requests (https://github.com/juhaku/utoipa/pull/1066)
* Refactor enums processing (https://github.com/juhaku/utoipa/pull/1059)
* Add support for real generics (https://github.com/juhaku/utoipa/pull/1034)
* Add support to define multiple operation methods (https://github.com/juhaku/utoipa/pull/1006)
* Feature openapi 31 (https://github.com/juhaku/utoipa/pull/981)
* Enhance OpenApi nesting with tags support (https://github.com/juhaku/utoipa/pull/932)
* allow for multiple req body content_type (https://github.com/juhaku/utoipa/pull/876)

## 4.3.1 - October 7 2024

### Changes

* Bump up `utoipa-gen` patch version
* change pub(super) enum to pub enum (#926) @cuiihaoo

## 4.3.0 - May 5 2024

### Added

* Add additional check to ensure generic type resolution is only for generics (https://github.com/juhaku/utoipa/pull/904)
* Add `default-features = false` to the optional axum dependency to avoid pulling in tokio in non-tokio environments (https://github.com/juhaku/utoipa/pull/874)

### Fixed

* Fix spelling (https://github.com/juhaku/utoipa/pull/846)

### Changed

* Seems like the zip_next is nowadays just zip again
* Update docs and next versions
* Skip 1st line in path macro description expansion (https://github.com/juhaku/utoipa/pull/881)
* Implement include_str! for tags (https://github.com/juhaku/utoipa/pull/893)

## 4.2.0 - Jan 9 2024

### Added

* Add support for specifying multiple security requirement keys (https://github.com/juhaku/utoipa/pull/813)

### Changed

* Update next versions
* Allowing utoipa/utoipa-swagger-ui successfully build on Windows and made path proc macro attribute more permissive (https://github.com/juhaku/utoipa/pull/830)
* Update Rocket v0.5 (https://github.com/juhaku/utoipa/pull/825)
* Update docs
* Path impl_for override. PathBuilder::path_from (https://github.com/juhaku/utoipa/pull/759)
* Support serde deny_unknown_fields (https://github.com/juhaku/utoipa/pull/816)
* Misc document improvements (https://github.com/juhaku/utoipa/pull/814)
* Hide Debug behind debug feature (https://github.com/juhaku/utoipa/pull/815)
* Axum 0.7 bindings (https://github.com/juhaku/utoipa/pull/807)

## 4.1.0 - Nov 13 2023

### Added

* feat: add HashSet and BTreeSet (https://github.com/juhaku/utoipa/pull/791)

### Changed

* Update next versions
* Support `#[serde(flatten)]` for maps. (https://github.com/juhaku/utoipa/pull/799)

## 4.0.0 - Oct 7 2023

### Added

* Add test for date types in actix params (https://github.com/juhaku/utoipa/pull/758)
* Add `decimal_float` feature. (https://github.com/juhaku/utoipa/pull/750)

### Changed

* Update next versions and dependencies
* Allow expression as macro arg (https://github.com/juhaku/utoipa/pull/762)
* enable required usage with schema_with attribute (https://github.com/juhaku/utoipa/pull/764)
* Feat std::collections::LinkedList as a known field type for schema (https://github.com/juhaku/utoipa/pull/748)
* Feat url type (https://github.com/juhaku/utoipa/pull/747)

## 3.5.0 - Aug 20 2023

### Added

* Add support for serde skip in `IntoParams` derive (https://github.com/juhaku/utoipa/pull/743)

### Changed

* Update next versions
* Support ULID (https://github.com/juhaku/utoipa/pull/733)

## 3.4.5 - Aug 3 2023

### Added

* Add more axum path parameter tests
* Add descriptions to 2 variants of complex enums  (https://github.com/juhaku/utoipa/pull/714)
* Add support for #[schema(default = )] on user-defined types (https://github.com/juhaku/utoipa/pull/712) (#713)

### Fixed

* Fix generics actix example (https://github.com/juhaku/utoipa/pull/716)
* Fix typos in doc (https://github.com/juhaku/utoipa/pull/709)

### Changed

* Update next versions
* allow and ignore #[doc(...)] tags in ToSchema derive (https://github.com/juhaku/utoipa/pull/708)
* Allow setting titles on all OpenApi Schema types and allow descriptions to propagate for UnnamedStructSchema (https://github.com/juhaku/utoipa/pull/694)

## 3.4.4 - Jul 23 2023

### Fixed

* Fix automatic request body (https://github.com/juhaku/utoipa/pull/701)

### Changed

* Update next versions

## 3.4.3 - Jul 23 2023

### Fixed

* Fix `Arc<T>` and `Rc<T>` and `SmallVec<[T]>` (https://github.com/juhaku/utoipa/pull/699)
* Fix broken link and enforce workspace resolver

## 3.4.2 - Jul 22 2023

### Added
* Added support for Arc fields to be treated like Box or RefCell (https://github.com/juhaku/utoipa/pull/690)
* Add support for deprecation using schema attribute (https://github.com/juhaku/utoipa/pull/688)
* Add enum path param test (https://github.com/juhaku/utoipa/pull/680)
* Add tests for uuid path params (https://github.com/juhaku/utoipa/pull/676)

### Fixed

* Fix `Option<Query<T>>` type support (https://github.com/juhaku/utoipa/pull/678)

### Changed

* Update next versions
* Disable automatic parameter recognition (https://github.com/juhaku/utoipa/pull/696)

## 3.4.1 - Jul 13 2023

### Fixed
* Fix utoipa-gen feature and update versions

## 3.4.0 Jul 13 2023

### Added

* Add automatic body recognition for rocket (https://github.com/juhaku/utoipa/pull/670)
* Add automatic type recognition for axum (https://github.com/juhaku/utoipa/pull/668)
* Add automatic query parameter recognition (https://github.com/juhaku/utoipa/pull/666)
* Add support for chrono::NaiveTime (https://github.com/juhaku/utoipa/pull/641)
* Add automatic request body recognition (https://github.com/juhaku/utoipa/pull/589)
* Add docs and tests for aliases (https://github.com/juhaku/utoipa/pull/587)
* Add basic auto response type support (https://github.com/juhaku/utoipa/pull/582)

### Fixed

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

## 3.3.0 - Apr 16 2023

### Added

* Add `indexmap` feature support for `TypeTree`

### Fixed

* Fix Schema as additional properties (https://github.com/juhaku/utoipa/pull/580)

### Changed

* Update next release versions
* Allow additional integer types (https://github.com/juhaku/utoipa/pull/575)
* Bump rocket to v0.5.0-rc.3 (https://github.com/juhaku/utoipa/pull/577)
* Allow value_type serde_json::Value (https://github.com/juhaku/utoipa/pull/568)
* Rename AdditionalProperties (https://github.com/juhaku/utoipa/pull/564)

## 3.2.1 - May 31 2023

### Changed

* Update next release versions (https://github.com/juhaku/utoipa/pull/555)
* Dont rely on listed serde_json crate

## 3.2.0 - May 28 2023

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

## 3.1.2 - May 20 2023

### Added

* Add support for double number format (https://github.com/juhaku/utoipa/pull/526)

### Changed

* Update next versions
* Make `Option` non-required & add `required` attr (https://github.com/juhaku/utoipa/pull/530)
* Remove needles ToTokens import
* Clean up & clippy lint
* Unify component schema tokenization (https://github.com/juhaku/utoipa/pull/525)

## 3.1.1 - May 16 2023

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

### Added
* Add support for serde `skip_serializing` (https://github.com/juhaku/utoipa/pull/438)
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

### Changed

* Refactor `ToResponse` trait (https://github.com/juhaku/utoipa/pull/460)
* Refactor to schema casting as (https://github.com/juhaku/utoipa/pull/459)
* Enhance generic aliases with lifetimes support (https://github.com/juhaku/utoipa/pull/458)
* Enhance path tuple argument support (https://github.com/juhaku/utoipa/pull/455)
* Update versions
* Improve docs (https://github.com/juhaku/utoipa/pull/444)
* Enhance responses derive support (https://github.com/juhaku/utoipa/pull/443)
* Feat/serde enum representation (https://github.com/juhaku/utoipa/pull/414)
* Enhance `ToResponse` implementation (https://github.com/juhaku/utoipa/pull/419)
* Address clippy lint
* Improve documentation
* Enhance repeated attributes support (https://github.com/juhaku/utoipa/pull/411)
* Make derive OpenApi server variable names LitStr
* Refactor `Type` to `TypeTree` (https://github.com/juhaku/utoipa/pull/408)
* Update `ToSchema` documentation
* Chore make `serde_json` mandatory dependency (https://github.com/juhaku/utoipa/pull/378)
* Feature http status codes (https://github.com/juhaku/utoipa/pull/376)
* Refactor some path derive with `IntoParmas` tests
* Chore refine description attribute (https://github.com/juhaku/utoipa/pull/373)
* cargo format
* Update to axum 0.6.0 (https://github.com/juhaku/utoipa/pull/369)
