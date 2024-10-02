# Changelog - utoipa

## Unreleased

### Added

* [8a5bb72](https://github.com/juhaku/utoipa/commit/8a5bb72) Add auto collect schemas for utoipa-axum (https://github.com/juhaku/utoipa/pull/1072)
* [e66b4ed](https://github.com/juhaku/utoipa/commit/e66b4ed) Add global config for `utiopa` (https://github.com/juhaku/utoipa/pull/1048)
* [f6d1c3d](https://github.com/juhaku/utoipa/commit/f6d1c3d) Add support for `links` in `#[utoipa::path]` (https://github.com/juhaku/utoipa/pull/1047)
* [06d539c](https://github.com/juhaku/utoipa/commit/06d539c) Add support for `termsOfService` to OpenApi derive (https://github.com/juhaku/utoipa/pull/1046)
* [9d778b0](https://github.com/juhaku/utoipa/commit/9d778b0) Add typos to CI (https://github.com/juhaku/utoipa/pull/1036)
* [5341dd0](https://github.com/juhaku/utoipa/commit/5341dd0) Add support for nullable schema map items (https://github.com/juhaku/utoipa/pull/1032)
* [af79ed6](https://github.com/juhaku/utoipa/commit/af79ed6) Add test for logical or security requirement
* [a85e3f4](https://github.com/juhaku/utoipa/commit/a85e3f4) Add paths support for routes! macro (https://github.com/juhaku/utoipa/pull/1023)
* [3225e4d](https://github.com/juhaku/utoipa/commit/3225e4d) Add extensions support for OpenApi (https://github.com/juhaku/utoipa/pull/1013)
* [9c79ef2](https://github.com/juhaku/utoipa/commit/9c79ef2) Add parsing support for non strict integers (https://github.com/juhaku/utoipa/pull/1012)
* [908d279](https://github.com/juhaku/utoipa/commit/908d279) Add `utoipa-axum` binding example and update docs (https://github.com/juhaku/utoipa/pull/1007)
* [a0db8b9](https://github.com/juhaku/utoipa/commit/a0db8b9) Add utoipa axum bindings (https://github.com/juhaku/utoipa/pull/1004)
* [55544ef](https://github.com/juhaku/utoipa/commit/55544ef) Add some deprecated attributes for `example` method
* [92cac85](https://github.com/juhaku/utoipa/commit/92cac85) Add support for inlined enum variants (https://github.com/juhaku/utoipa/pull/963)
* [b941d64](https://github.com/juhaku/utoipa/commit/b941d64) Add extensions for schemas (https://github.com/juhaku/utoipa/pull/983)
* [674d0b9](https://github.com/juhaku/utoipa/commit/674d0b9) Add `description` attribute on `ToSchema` (https://github.com/juhaku/utoipa/pull/949)
* [f7750fc](https://github.com/juhaku/utoipa/commit/f7750fc) Add support for description and summary overriding (https://github.com/juhaku/utoipa/pull/948)
* [c63407d](https://github.com/juhaku/utoipa/commit/c63407d) Add vendored Swagger UI for utoipa (https://github.com/juhaku/utoipa/pull/941)
* [139c035](https://github.com/juhaku/utoipa/commit/139c035) Add missing windows check to utoipa-swagger-ui build.rs
* [ea59c38](https://github.com/juhaku/utoipa/commit/ea59c38) Address clippy lints and refactor serde parsing (https://github.com/juhaku/utoipa/pull/931)
* [97bc507](https://github.com/juhaku/utoipa/commit/97bc507) Add nest `OpenApi` support (https://github.com/juhaku/utoipa/pull/930)
* [86339ec](https://github.com/juhaku/utoipa/commit/86339ec) Add `merge_from` method for chainable merge (https://github.com/juhaku/utoipa/pull/924)
* [9f8ebf3](https://github.com/juhaku/utoipa/commit/9f8ebf3) Add support for addtional tags via `tags` (https://github.com/juhaku/utoipa/pull/916)

### Fixed

* [abd3572](https://github.com/juhaku/utoipa/commit/abd3572) Fix brokent doc links
* [4147119](https://github.com/juhaku/utoipa/commit/4147119) Fix allow response `content_type` without schema (https://github.com/juhaku/utoipa/pull/1073)
* [2d81c9b](https://github.com/juhaku/utoipa/commit/2d81c9b) Fix testing without explicit features (https://github.com/juhaku/utoipa/pull/1041)
* [4422cee](https://github.com/juhaku/utoipa/commit/4422cee) Fix building utoipa-rapidoc & utoipa-scalar (https://github.com/juhaku/utoipa/pull/1039)
* [fcdb5db](https://github.com/juhaku/utoipa/commit/fcdb5db) Fix building utoipa-axum & utoipa-swagger-ui (https://github.com/juhaku/utoipa/pull/1038)
* [ed255a1](https://github.com/juhaku/utoipa/commit/ed255a1) Fix negative value parsing on schema attributes (https://github.com/juhaku/utoipa/pull/1031)
* [11058c6](https://github.com/juhaku/utoipa/commit/11058c6) Fix utoipa-axum project description
* [8948d34](https://github.com/juhaku/utoipa/commit/8948d34) Fix parameter inline for tuple path params (https://github.com/juhaku/utoipa/pull/1014)
* [bcc4fca](https://github.com/juhaku/utoipa/commit/bcc4fca) Fix some typos
* [f2a7143](https://github.com/juhaku/utoipa/commit/f2a7143) Fix default tag logic for paths (https://github.com/juhaku/utoipa/pull/1002)
* [a5695f0](https://github.com/juhaku/utoipa/commit/a5695f0) Fix utoipa-swagger-ui-vendored crates link
* [d559882](https://github.com/juhaku/utoipa/commit/d559882) Fixed documentation spelling mistake (https://github.com/juhaku/utoipa/pull/999)
* [2c76479](https://github.com/juhaku/utoipa/commit/2c76479) Fix RapiDoc with empty URL panic on axum (https://github.com/juhaku/utoipa/pull/997)
* [5e780f1](https://github.com/juhaku/utoipa/commit/5e780f1) Fix respect `required` attribute (https://github.com/juhaku/utoipa/pull/990)
* [c774742](https://github.com/juhaku/utoipa/commit/c774742) Fix samples in documentation when with_openapi is used (https://github.com/juhaku/utoipa/pull/988)
* [2406c75](https://github.com/juhaku/utoipa/commit/2406c75) Fix openapi serialized twice when served with Rocket (https://github.com/juhaku/utoipa/pull/987)
* [cc78e38](https://github.com/juhaku/utoipa/commit/cc78e38) Fix find actual request body TypeTree (https://github.com/juhaku/utoipa/pull/977)
* [403d716](https://github.com/juhaku/utoipa/commit/403d716) Fix summary / description split on empty lines (https://github.com/juhaku/utoipa/pull/947)
* [360e23e](https://github.com/juhaku/utoipa/commit/360e23e) Fix docs.rs build
* [28cf85c](https://github.com/juhaku/utoipa/commit/28cf85c) Fix compile error propagation (https://github.com/juhaku/utoipa/pull/929)
* [48342ae](https://github.com/juhaku/utoipa/commit/48342ae) Fix tuple params missing features (https://github.com/juhaku/utoipa/pull/928)

### Changed

* [ac13f48](https://github.com/juhaku/utoipa/commit/ac13f48) Chore unify request body and ext request body (https://github.com/juhaku/utoipa/pull/1067)
* [e0c5aa7](https://github.com/juhaku/utoipa/commit/e0c5aa7) Refactor structs processing (https://github.com/juhaku/utoipa/pull/1060)
* [576f6c1](https://github.com/juhaku/utoipa/commit/576f6c1) Update README.md
* [26d4084](https://github.com/juhaku/utoipa/commit/26d4084) Update README.md
* [78656b6](https://github.com/juhaku/utoipa/commit/78656b6) Always use system `curl` by default (https://github.com/juhaku/utoipa/pull/1045)
* [a985d8c](https://github.com/juhaku/utoipa/commit/a985d8c) Remove Redirect Causing Invalid URIs for Swagger UIs Server on / (https://github.com/juhaku/utoipa/pull/1043)
* [18d004a](https://github.com/juhaku/utoipa/commit/18d004a) Disable unused default features of rust_decimal (https://github.com/juhaku/utoipa/pull/1029)
* [8d8ff6e](https://github.com/juhaku/utoipa/commit/8d8ff6e) Update utoipa-axum version
* [c36f877](https://github.com/juhaku/utoipa/commit/c36f877) Use fs::read to overwrite swagger UI contents (https://github.com/juhaku/utoipa/pull/1022)
* [b473b99](https://github.com/juhaku/utoipa/commit/b473b99) Make referenced schemas required (https://github.com/juhaku/utoipa/pull/1018)
* [90ec7a6](https://github.com/juhaku/utoipa/commit/90ec7a6) Enhance `utoipa-axum` bindings (https://github.com/juhaku/utoipa/pull/1017)
* [57ba3ba](https://github.com/juhaku/utoipa/commit/57ba3ba) Update next beta versions
* [04c490d](https://github.com/juhaku/utoipa/commit/04c490d) Chore refactor `OpenApi` derive macro (https://github.com/juhaku/utoipa/pull/1011)
* [509e8e7](https://github.com/juhaku/utoipa/commit/509e8e7) Chore fix clippy lint (https://github.com/juhaku/utoipa/pull/1010)
* [7882cb2](https://github.com/juhaku/utoipa/commit/7882cb2) Chore refactor Name trait usage in features (https://github.com/juhaku/utoipa/pull/1009)
* [708645a](https://github.com/juhaku/utoipa/commit/708645a) Chore refactor features (https://github.com/juhaku/utoipa/pull/1008)
* [1af4ad4](https://github.com/juhaku/utoipa/commit/1af4ad4) Chore update docs and relax `url` version (https://github.com/juhaku/utoipa/pull/1001)
* [89c288b](https://github.com/juhaku/utoipa/commit/89c288b) Bump up versions (https://github.com/juhaku/utoipa/pull/998)
* [67f04b3](https://github.com/juhaku/utoipa/commit/67f04b3) Clean up some unused fields
* [2088259](https://github.com/juhaku/utoipa/commit/2088259) fix(utoipa-gen): remove unnecessary allocation with to_string in expanded code (https://github.com/juhaku/utoipa/pull/982)
* [50dbec1](https://github.com/juhaku/utoipa/commit/50dbec1) Bump up to next alplha
* [d020f92](https://github.com/juhaku/utoipa/commit/d020f92) Update versions
* [68172bf](https://github.com/juhaku/utoipa/commit/68172bf) Update utoipa-swagger-ui vendored dependency
* [91a98ef](https://github.com/juhaku/utoipa/commit/91a98ef) Update utoipa-swagger-ui-vendored version
* [2ddc279](https://github.com/juhaku/utoipa/commit/2ddc279) Include res for crates
* [164c161](https://github.com/juhaku/utoipa/commit/164c161) Update min Rust version and utoipa-swagger-ui version
* [72218c9](https://github.com/juhaku/utoipa/commit/72218c9) Update `utoipa-swagger-ui` versions (https://github.com/juhaku/utoipa/pull/938)
* [335407b](https://github.com/juhaku/utoipa/commit/335407b) Use CARGO_HTTP_CAINFO CA file in build script if present (https://github.com/juhaku/utoipa/pull/935)
* [5530d29](https://github.com/juhaku/utoipa/commit/5530d29) Clean up imports for utoipa-gen
* [7563a72](https://github.com/juhaku/utoipa/commit/7563a72) change pub(super) enum to pub enum (https://github.com/juhaku/utoipa/pull/926)
* [19ff135](https://github.com/juhaku/utoipa/commit/19ff135) Improve file:// url parsing (https://github.com/juhaku/utoipa/pull/925)
* [f03e7d5](https://github.com/juhaku/utoipa/commit/f03e7d5) Migrate out from proc macro error (https://github.com/juhaku/utoipa/pull/920)
* [272ceb8](https://github.com/juhaku/utoipa/commit/272ceb8) Make SWAGGER_UI_DOWNLOAD_URL support file:// urls (https://github.com/juhaku/utoipa/pull/923)

### Breaking

* [8d5149f](https://github.com/juhaku/utoipa/commit/8d5149f) Auto collect tuple responses schema references (https://github.com/juhaku/utoipa/pull/1071)
* [8e5b818](https://github.com/juhaku/utoipa/commit/8e5b818) Add explicit `Extensions` type (https://github.com/juhaku/utoipa/pull/1062)
* [5a06616](https://github.com/juhaku/utoipa/commit/5a06616) Implement automatic schema collection for requests (https://github.com/juhaku/utoipa/pull/1066)
* [e13cfe1](https://github.com/juhaku/utoipa/commit/e13cfe1) Refactor enums processing (https://github.com/juhaku/utoipa/pull/1059)
* [f13d3d3](https://github.com/juhaku/utoipa/commit/f13d3d3) Add support for real generics (https://github.com/juhaku/utoipa/pull/1034)
* [b0ae6ef](https://github.com/juhaku/utoipa/commit/b0ae6ef) Chore change the operations implementation. (https://github.com/juhaku/utoipa/pull/1026)
* [11c909b](https://github.com/juhaku/utoipa/commit/11c909b) Add macros feature flag (https://github.com/juhaku/utoipa/pull/1015)
* [69dfbbc](https://github.com/juhaku/utoipa/commit/69dfbbc) Add support to define mulitple operation methods (https://github.com/juhaku/utoipa/pull/1006)
* [ae6cedd](https://github.com/juhaku/utoipa/commit/ae6cedd) Feature openapi 31 (https://github.com/juhaku/utoipa/pull/981)
* [ffcd202](https://github.com/juhaku/utoipa/commit/ffcd202) Enhance utoipa-swagger-ui build (https://github.com/juhaku/utoipa/pull/936)
* [b22eb1a](https://github.com/juhaku/utoipa/commit/b22eb1a) Enhance OpenApi nesting with tags support (https://github.com/juhaku/utoipa/pull/932)
* [2661057](https://github.com/juhaku/utoipa/commit/2661057) allow for multiple req body content_type (https://github.com/juhaku/utoipa/pull/876)

## 4.2.3 - May 7 2024

### Changed

* [e0718de](https://github.com/juhaku/utoipa/commit/e0718de) Update utoipa version
* [8594735](https://github.com/juhaku/utoipa/commit/8594735) Make reqwest to use rustls instead of openssl (https://github.com/juhaku/utoipa/pull/912)
* [09eb5ab](https://github.com/juhaku/utoipa/commit/09eb5ab) Have OpenApi::merge consider operations/methods as well as paths (https://github.com/juhaku/utoipa/pull/910)

## 4.2.2 - May 7 2024

### Changed

* [2de0fb7](https://github.com/juhaku/utoipa/commit/2de0fb7) Fix utoipa-gen dependency version (https://github.com/juhaku/utoipa/pull/909)

## 4.2.1 - May 5 2024

### Added

* [9713b26](https://github.com/juhaku/utoipa/commit/9713b26) Add additional check to ensure generic type resolution is only for generics (https://github.com/juhaku/utoipa/pull/904)
* [613b3ad](https://github.com/juhaku/utoipa/commit/613b3ad) Add crate for serving Scalar via utoipa (https://github.com/juhaku/utoipa/pull/892)
* [4b32ba9](https://github.com/juhaku/utoipa/commit/4b32ba9) Add `default-features = false` to the optional axum dependency to avoid pulling in tokio in non-tokio environments (https://github.com/juhaku/utoipa/pull/874) 
* [776f753](https://github.com/juhaku/utoipa/commit/776f753) Add flex to `utoipa-swagger-ui` build (https://github.com/juhaku/utoipa/pull/845)

### Changed

* [19fb23c](https://github.com/juhaku/utoipa/commit/19fb23c) Seems like the zip_next is nowadays just zip again
* [c907d43](https://github.com/juhaku/utoipa/commit/c907d43) Update docs and next versions
* [becc132](https://github.com/juhaku/utoipa/commit/becc132) Use same licences for scalar as well
* [926a5e8](https://github.com/juhaku/utoipa/commit/926a5e8) Update default Swagger UI version (https://github.com/juhaku/utoipa/pull/905)
* [5aa9749](https://github.com/juhaku/utoipa/commit/5aa9749) Skip 1st line in path macro description expansion (https://github.com/juhaku/utoipa/pull/881)
* [365469f](https://github.com/juhaku/utoipa/commit/365469f) Implement include_str! for tags (https://github.com/juhaku/utoipa/pull/893)
* [4d798bc](https://github.com/juhaku/utoipa/commit/4d798bc) Replace `zip` with `zip_next` (https://github.com/juhaku/utoipa/pull/889)
* [7b74942](https://github.com/juhaku/utoipa/commit/7b74942) Refactor `ReDoc` to take `Cow<'static, str>` instead of borrowed `str` (https://github.com/juhaku/utoipa/pull/869)
* [5c6b0e2](https://github.com/juhaku/utoipa/commit/5c6b0e2) Refactor RapiDoc to take `Cow<'static, str>` instead of borrowed `str` (https://github.com/juhaku/utoipa/pull/867)
* [8639185](https://github.com/juhaku/utoipa/commit/8639185) Fix spelling (https://github.com/juhaku/utoipa/pull/846)

## 4.2.0 - Jan 9 2024

### Added

* [7493c33](https://github.com/juhaku/utoipa/commit/7493c33) Add support for specifying multiple security requirement keys (https://github.com/juhaku/utoipa/pull/813)

### Changed

* [f7cae03](https://github.com/juhaku/utoipa/commit/f7cae03) Update next versions
* [fe229e2](https://github.com/juhaku/utoipa/commit/fe229e2) Allowing utoipa/utoipa-swagger-ui successfully build on Windows and made path proc macro attribute more permissive (https://github.com/juhaku/utoipa/pull/830)
* [d437919](https://github.com/juhaku/utoipa/commit/d437919) Update Rocket v0.5 (https://github.com/juhaku/utoipa/pull/825)
* [3e5a635](https://github.com/juhaku/utoipa/commit/3e5a635) Generate embed code instead of using interpolation (https://github.com/juhaku/utoipa/pull/828)
* [1ea9cf8](https://github.com/juhaku/utoipa/commit/1ea9cf8) Update docs
* [1b9c39b](https://github.com/juhaku/utoipa/commit/1b9c39b) Path impl_for override. PathBuilder::path_from (https://github.com/juhaku/utoipa/pull/759)
* [a968ced](https://github.com/juhaku/utoipa/commit/a968ced) fix: fix typo (https://github.com/juhaku/utoipa/pull/822)
* [f965165](https://github.com/juhaku/utoipa/commit/f965165) Support serde deny_unknown_fields (https://github.com/juhaku/utoipa/pull/816)
* [7e49344](https://github.com/juhaku/utoipa/commit/7e49344) Misc document improvements (https://github.com/juhaku/utoipa/pull/814)
* [beb68fa](https://github.com/juhaku/utoipa/commit/beb68fa) Hide Debug behind debug feature (https://github.com/juhaku/utoipa/pull/815)
* [35f32b1](https://github.com/juhaku/utoipa/commit/35f32b1) Update next versions
* [93dfaf1](https://github.com/juhaku/utoipa/commit/93dfaf1) Axum 0.7 bindings (https://github.com/juhaku/utoipa/pull/807)

## 4.1.0 - Nov 13 2023

### Added

* [2f89c69](https://github.com/juhaku/utoipa/commit/2f89c69) feat: add HashSet and BTreeSet (https://github.com/juhaku/utoipa/pull/791)
* [f43efc7](https://github.com/juhaku/utoipa/commit/f43efc7) add openapi extensions "x-tokenName" (https://github.com/juhaku/utoipa/pull/763)

### Changed

* [048d898](https://github.com/juhaku/utoipa/commit/048d898) Update next versions
* [5d96e30](https://github.com/juhaku/utoipa/commit/5d96e30) Support `#[serde(flatten)]` for maps. (https://github.com/juhaku/utoipa/pull/799)
* [50db1b0](https://github.com/juhaku/utoipa/commit/50db1b0) Update utoipa versions

## 4.0.0 - Oct 7 2023

### Added

* [6dc73eb](https://github.com/juhaku/utoipa/commit/6dc73eb) Add Discriminator mapping (https://github.com/juhaku/utoipa/pull/752)
* [d7280eb](https://github.com/juhaku/utoipa/commit/d7280eb) Add test for date types in actix params (https://github.com/juhaku/utoipa/pull/758)
* [b1ce2d0](https://github.com/juhaku/utoipa/commit/b1ce2d0) Add `decimal_float` feature. (https://github.com/juhaku/utoipa/pull/750)

### Fixed

* [4e10648](https://github.com/juhaku/utoipa/commit/4e10648) Fix: panic on missing trailing / in rocket environment (https://github.com/juhaku/utoipa/pull/645) (#757)

### Changed

* [15053c5](https://github.com/juhaku/utoipa/commit/15053c5) Update next versions and dependencies
* [164e0d3](https://github.com/juhaku/utoipa/commit/164e0d3) enable required usage with schema_with attribute (https://github.com/juhaku/utoipa/pull/764)
* [1b3479e](https://github.com/juhaku/utoipa/commit/1b3479e) Allow additionalProperties to be an array (https://github.com/juhaku/utoipa/pull/756)
* [1443ec4](https://github.com/juhaku/utoipa/commit/1443ec4) Feat std::collections::LinkedList as a known field type for schema (https://github.com/juhaku/utoipa/pull/748)
* [2eecc9a](https://github.com/juhaku/utoipa/commit/2eecc9a) Feat url type (https://github.com/juhaku/utoipa/pull/747)

### Breaking

* [a235161](https://github.com/juhaku/utoipa/commit/a235161) Allow expression as macro arg (https://github.com/juhaku/utoipa/pull/762)

## 3.5.0 - Aug 20 2023

### Added

* [5fb25fa](https://github.com/juhaku/utoipa/commit/5fb25fa) Add support for serde skip in `IntoParams` derive (https://github.com/juhaku/utoipa/pull/743)
* [56b6326](https://github.com/juhaku/utoipa/commit/56b6326) Add rapidoc support (https://github.com/juhaku/utoipa/pull/723)
* [6baa9b0](https://github.com/juhaku/utoipa/commit/6baa9b0) Add redoc support for utoipa. (https://github.com/juhaku/utoipa/pull/720)

### Changed

* [387a97c](https://github.com/juhaku/utoipa/commit/387a97c) Update next versions
* [0c49940](https://github.com/juhaku/utoipa/commit/0c49940) Support ULID (https://github.com/juhaku/utoipa/pull/733)
* [30551f6](https://github.com/juhaku/utoipa/commit/30551f6) Update utoipa-swagger-ui version
* [f9d26f6](https://github.com/juhaku/utoipa/commit/f9d26f6) Update Swagger UI to 5.3.1
* [e5f7f70](https://github.com/juhaku/utoipa/commit/e5f7f70) Update README.md docs

## 3.4.4 - Aug 3 2023

### Added

* [7cc90b1](https://github.com/juhaku/utoipa/commit/7cc90b1) Add more axum path parameter tests
* [cea4c50](https://github.com/juhaku/utoipa/commit/cea4c50) Add descriptions to 2 variants of complex enums  (https://github.com/juhaku/utoipa/pull/714)
* [b1a5d4f](https://github.com/juhaku/utoipa/commit/b1a5d4f) Add support for #[schema(default = )] on user-defined types (https://github.com/juhaku/utoipa/pull/712) (#713)
* [727f880](https://github.com/juhaku/utoipa/commit/727f880) Adding "AnyOf" branch for Schema (https://github.com/juhaku/utoipa/pull/706)

### Fixed

* [1dccaf4](https://github.com/juhaku/utoipa/commit/1dccaf4) Fix generics actix example (https://github.com/juhaku/utoipa/pull/716)
* [338c413](https://github.com/juhaku/utoipa/commit/338c413) Fix typos in doc (https://github.com/juhaku/utoipa/pull/709)

### Changed

* [a9f4797](https://github.com/juhaku/utoipa/commit/a9f4797) Update next versions
* [2c811ee](https://github.com/juhaku/utoipa/commit/2c811ee) allow and ignore #[doc(...)] tags in ToSchema derive (https://github.com/juhaku/utoipa/pull/708)
* [97d3617](https://github.com/juhaku/utoipa/commit/97d3617) Allow setting titles on all OpenApi Schema types and allow descriptions to propagate for UnnamedStructSchema (https://github.com/juhaku/utoipa/pull/694)

## 3.4.3 - Jul 23 2023

### Fixed

* [fe3b42d](https://github.com/juhaku/utoipa/commit/fe3b42d) Fix automatic request body (https://github.com/juhaku/utoipa/pull/701)

### Changed

* [bcae7bc](https://github.com/juhaku/utoipa/commit/bcae7bc) Update next versions

## 3.4.2 - Jul 23 2023

### Fixed

* [c69bb26](https://github.com/juhaku/utoipa/commit/c69bb26) Fix `Arc<T>` and `Rc<T>` and `SmallVec<[T]>` (https://github.com/juhaku/utoipa/pull/699)
* [341dd39](https://github.com/juhaku/utoipa/commit/341dd39) Fix broken link and enforce workspace resolver

## 3.4.1 - Jul 22 2023

### Added

* [0073541](https://github.com/juhaku/utoipa/commit/0073541) Add support for deprecation using schema attribute (https://github.com/juhaku/utoipa/pull/688)
* [a334fda](https://github.com/juhaku/utoipa/commit/a334fda) Add enum path param test (https://github.com/juhaku/utoipa/pull/680)
* [3732779](https://github.com/juhaku/utoipa/commit/3732779) Add tests for uuid path params (https://github.com/juhaku/utoipa/pull/676)

### Fixed

* [99020a9](https://github.com/juhaku/utoipa/commit/99020a9) Fix `Option<Query<T>>` type support (https://github.com/juhaku/utoipa/pull/678)

### Changed 

* [23f4a83](https://github.com/juhaku/utoipa/commit/23f4a83) Update next versions
* [73fd3ea](https://github.com/juhaku/utoipa/commit/73fd3ea) Disable automatic parameter recognition (https://github.com/juhaku/utoipa/pull/696)
* [8424b97](https://github.com/juhaku/utoipa/commit/8424b97) Added support for Arc fields to be treated like Box or RefCell (https://github.com/juhaku/utoipa/pull/690)

## 3.4.0 - Jul 13 2023

### Added

* [90b875d](https://github.com/juhaku/utoipa/commit/90b875d) Add automatic body recognition for rocket (https://github.com/juhaku/utoipa/pull/670)
* [41d8f58](https://github.com/juhaku/utoipa/commit/41d8f58) Add automatic type recognition for axum (https://github.com/juhaku/utoipa/pull/668)
* [d008ff4](https://github.com/juhaku/utoipa/commit/d008ff4) Add automatic query parameter recognition (https://github.com/juhaku/utoipa/pull/666)
* [d9c4702](https://github.com/juhaku/utoipa/commit/d9c4702) Add support for chrono::NaiveTime (https://github.com/juhaku/utoipa/pull/641)
* [fcdcc19](https://github.com/juhaku/utoipa/commit/fcdcc19) Add `preserve_path_order` cargo feature docs (https://github.com/juhaku/utoipa/pull/614)
* [888fc72](https://github.com/juhaku/utoipa/commit/888fc72) Add automatic request body recognition (https://github.com/juhaku/utoipa/pull/589)
* [6c89f81](https://github.com/juhaku/utoipa/commit/6c89f81) Add docs and tests for aliases (https://github.com/juhaku/utoipa/pull/587)
* [c6eecf4](https://github.com/juhaku/utoipa/commit/c6eecf4) Add basic auto response type support (https://github.com/juhaku/utoipa/pull/582)
* [5d0205c](https://github.com/juhaku/utoipa/commit/5d0205c) Add preserve_path_order feature (https://github.com/juhaku/utoipa/pull/612)

### Fixed

* [588ff69](https://github.com/juhaku/utoipa/commit/588ff69) Fix utoipa-gen feature and update versions
* [eed338b](https://github.com/juhaku/utoipa/commit/eed338b) Fix broken links (https://github.com/juhaku/utoipa/pull/669)
* [970e10f](https://github.com/juhaku/utoipa/commit/970e10f) Fix tests for feature non_strict_integers (https://github.com/juhaku/utoipa/pull/619)

### Changed

* [6c2ca20](https://github.com/juhaku/utoipa/commit/6c2ca20) Update next versions
* [2979ce9](https://github.com/juhaku/utoipa/commit/2979ce9) Rename `auto_types` feature flag (https://github.com/juhaku/utoipa/pull/665)
* [7cf45ce](https://github.com/juhaku/utoipa/commit/7cf45ce) Chore add more feature flag checks for auto types
* [1774bb7](https://github.com/juhaku/utoipa/commit/1774bb7) Remove `type: object` restriction in empty() (https://github.com/juhaku/utoipa/pull/648)
* [f8c6d07](https://github.com/juhaku/utoipa/commit/f8c6d07) exclude const generic arguments from generic_types (https://github.com/juhaku/utoipa/pull/627)
* [16bec9d](https://github.com/juhaku/utoipa/commit/16bec9d) Make sure to parse a comma token after the status in IntoResponses (https://github.com/juhaku/utoipa/pull/630)
* [e6418ff](https://github.com/juhaku/utoipa/commit/e6418ff) Omit decimal zeros when serializing minimum/maximum/multiple (https://github.com/juhaku/utoipa/pull/618)
* [b59ee09](https://github.com/juhaku/utoipa/commit/b59ee09) Correct `with_schema` to `schema_with` in docs (https://github.com/juhaku/utoipa/pull/586)
* [d007625](https://github.com/juhaku/utoipa/commit/d007625) Improve docs for examples (https://github.com/juhaku/utoipa/pull/584)
* [0cf37c0](https://github.com/juhaku/utoipa/commit/0cf37c0) Use swagger-ui v4.18.2 (https://github.com/juhaku/utoipa/pull/585)

## 3.3.0 - Apr 16 2023

### Added

* [892bd61](https://github.com/juhaku/utoipa/commit/892bd61) Add more known formats (https://github.com/juhaku/utoipa/pull/571)
* [b75fa2d](https://github.com/juhaku/utoipa/commit/b75fa2d) Add `indexmap` feature support for `TypeTree`

### Fixed

* [1abced1](https://github.com/juhaku/utoipa/commit/1abced1) Fix Schema as additional properties (https://github.com/juhaku/utoipa/pull/580)
* [bab7960](https://github.com/juhaku/utoipa/commit/bab7960) Fix `preserve_order` feature (https://github.com/juhaku/utoipa/pull/562)

### Changed

* [89b809e](https://github.com/juhaku/utoipa/commit/89b809e) Update next release versions
* [96feca3](https://github.com/juhaku/utoipa/commit/96feca3) feat: Support deserializing other versions in 3.0.x (https://github.com/juhaku/utoipa/pull/578)
* [dc0cf3c](https://github.com/juhaku/utoipa/commit/dc0cf3c) Allow additional integer types (https://github.com/juhaku/utoipa/pull/575)
* [08acfa2](https://github.com/juhaku/utoipa/commit/08acfa2) Bump rocket to v0.5.0-rc.3 (https://github.com/juhaku/utoipa/pull/577)
* [3e6852d](https://github.com/juhaku/utoipa/commit/3e6852d) feat: Allow default value on Content::examples (https://github.com/juhaku/utoipa/pull/579)
* [c0c1470](https://github.com/juhaku/utoipa/commit/c0c1470) Allow value_type serde_json::Value (https://github.com/juhaku/utoipa/pull/568)
* [c0aead7](https://github.com/juhaku/utoipa/commit/c0aead7) Rename AdditionalProperites->AdditionalProperties (https://github.com/juhaku/utoipa/pull/564)
* [4aa305c](https://github.com/juhaku/utoipa/commit/4aa305c) Cargo format
* [cf5e9aa](https://github.com/juhaku/utoipa/commit/cf5e9aa) Update utoipa-swagger-ui version

## 3.2.1 - Mar 31 2023

### Changed

* [632437a](https://github.com/juhaku/utoipa/commit/632437a) Update next release versions (https://github.com/juhaku/utoipa/pull/555)
* [36bdfcc](https://github.com/juhaku/utoipa/commit/36bdfcc) Rename /api-doc/ to /api-docs
* [a499c64](https://github.com/juhaku/utoipa/commit/a499c64) Don't rely on listed serde_json crate


## 3.2.0 - Mar 28 2023

### Added

* [282c1b3](https://github.com/juhaku/utoipa/commit/282c1b3) Add support for partial schema (https://github.com/juhaku/utoipa/pull/544)
* [fed0226](https://github.com/juhaku/utoipa/commit/fed0226) Add tuple support for component schema (https://github.com/juhaku/utoipa/pull/541)
* [b2e99a8](https://github.com/juhaku/utoipa/commit/b2e99a8) Add missing enum variant examples (https://github.com/juhaku/utoipa/pull/538)
* [9d483a3](https://github.com/juhaku/utoipa/commit/9d483a3) Add support for auto-populating field default values (https://github.com/juhaku/utoipa/pull/533)

### Fixed

* [7b505fb](https://github.com/juhaku/utoipa/commit/7b505fb) Fix untagged enum unit variant support (https://github.com/juhaku/utoipa/pull/545)
* [2deda0a](https://github.com/juhaku/utoipa/commit/2deda0a) bugfix: use `map()` instead of `unwrap()` (https://github.com/juhaku/utoipa/pull/536)

### Changed

* [dcb15d3](https://github.com/juhaku/utoipa/commit/dcb15d3) Update next release versions
* [1d26a65](https://github.com/juhaku/utoipa/commit/1d26a65) Refactor alises support on `ToSchema` derive (https://github.com/juhaku/utoipa/pull/546)
* [ee88c75](https://github.com/juhaku/utoipa/commit/ee88c75) Upgrade to syn2 (https://github.com/juhaku/utoipa/pull/542)

## 3.1.2 - Mar 20 2023

### Added

* [84e6e68](https://github.com/juhaku/utoipa/commit/84e6e68) Add support for double number format (https://github.com/juhaku/utoipa/pull/526)

### Changed

* [323b155](https://github.com/juhaku/utoipa/commit/323b155) Update next versions
* [61046d1](https://github.com/juhaku/utoipa/commit/61046d1) Make `Option` non-required & add `required` attr (https://github.com/juhaku/utoipa/pull/530)
* [d399280](https://github.com/juhaku/utoipa/commit/d399280) Remove needles ToTokens import
* [43d3457](https://github.com/juhaku/utoipa/commit/43d3457) Clean up & clippy lint
* [f7dfff8](https://github.com/juhaku/utoipa/commit/f7dfff8) Unify component schema tokenization (https://github.com/juhaku/utoipa/pull/525)

## 3.1.1 - Mar 16 2023

### Added

* [53b96c3](https://github.com/juhaku/utoipa/commit/53b96c3) Add missing `As` attribute to complex enum (https://github.com/juhaku/utoipa/pull/516)
* [3ebf997](https://github.com/juhaku/utoipa/commit/3ebf997) Add support for chrono NaiveDateTime (https://github.com/juhaku/utoipa/pull/514)

### Fixed

* [9ab1836](https://github.com/juhaku/utoipa/commit/9ab1836) Fix empty contact creation (https://github.com/juhaku/utoipa/pull/517)

### Changed

* [195be49](https://github.com/juhaku/utoipa/commit/195be49) Update next versions
* [b4e11dc](https://github.com/juhaku/utoipa/commit/b4e11dc) Remove superfluous `deprecated` path attribute (https://github.com/juhaku/utoipa/pull/520)
* [44cd43e](https://github.com/juhaku/utoipa/commit/44cd43e) Cargo format
* [cd22c7e](https://github.com/juhaku/utoipa/commit/cd22c7e) Make unsigned integers implicityly minimum zero (https://github.com/juhaku/utoipa/pull/515)

## 3.1.0 - Mar 10 2023

### Added

* [4cb0b7e](https://github.com/juhaku/utoipa/commit/4cb0b7e) Add support for external OpenAPI docs (https://github.com/juhaku/utoipa/pull/502)
* [159d490](https://github.com/juhaku/utoipa/commit/159d490) Add full support for nullable field detection (https://github.com/juhaku/utoipa/pull/498)
* [746431d](https://github.com/juhaku/utoipa/commit/746431d) Add support for free form additional properties (https://github.com/juhaku/utoipa/pull/495)

### Fixed

* [2e501eb](https://github.com/juhaku/utoipa/commit/2e501eb) Fix nullable ref schema inline (https://github.com/juhaku/utoipa/pull/510)
* [1fbe015](https://github.com/juhaku/utoipa/commit/1fbe015) Fix nullable ref schema (https://github.com/juhaku/utoipa/pull/509)

### Changed

* [ddd138e](https://github.com/juhaku/utoipa/commit/ddd138e) Update next versions

## 3.0.3 - Feb 19 2023

### Added

* [3705e4e](https://github.com/juhaku/utoipa/commit/3705e4e) Add description support for object field. (https://github.com/juhaku/utoipa/pull/492)

### Fixed

* [937db4c](https://github.com/juhaku/utoipa/commit/937db4c) Fix clippy lint
* [959f7cb](https://github.com/juhaku/utoipa/commit/959f7cb) Fix function argument support for `#[utoipa::path]` (https://github.com/juhaku/utoipa/pull/489)
* [3996389](https://github.com/juhaku/utoipa/commit/3996389) Fix parsed version in info being ignored (https://github.com/juhaku/utoipa/pull/485)

### Changed

* [2d1f48d](https://github.com/juhaku/utoipa/commit/2d1f48d) Update next versions
* [e06d1ef](https://github.com/juhaku/utoipa/commit/e06d1ef) Improve description support on `ToSchema` fields (https://github.com/juhaku/utoipa/pull/490)
* [9098668](https://github.com/juhaku/utoipa/commit/9098668) Update OpenApi derive docs and tests

## 3.0.2 - Feb 10 2023

### Added

* [c4564ce](https://github.com/juhaku/utoipa/commit/c4564ce) Add support for unit type `()` (https://github.com/juhaku/utoipa/pull/464)

### Changed

* [fe39928](https://github.com/juhaku/utoipa/commit/fe39928) Update next versions
* [2986e5a](https://github.com/juhaku/utoipa/commit/2986e5a) Enhance unit type support (https://github.com/juhaku/utoipa/pull/476)
* [9124559](https://github.com/juhaku/utoipa/commit/9124559) Support arbitrary exprs in operation_id (https://github.com/juhaku/utoipa/pull/472)

## 3.0.1 - Jan 29 2023

### Fixed

* [6190978](https://github.com/juhaku/utoipa/commit/6190978) Fix explicit lifetimes for consts (https://github.com/juhaku/utoipa/pull/467)

### Changed

* [96acebf](https://github.com/juhaku/utoipa/commit/96acebf) Update next versions

## 3.0.0 - Jan 26 2023

Migration guide: https://github.com/juhaku/utoipa/discussions/456

### Added
* [b167838](https://github.com/juhaku/utoipa/commit/b167838) Add support for serde `skip_serializing` (https://github.com/juhaku/utoipa/pull/438)
* [2a5c09d](https://github.com/juhaku/utoipa/commit/2a5c09d) Add `preserve_order` feature to preserve property order during serialization (https://github.com/juhaku/utoipa/pull/436)
* [4a08316](https://github.com/juhaku/utoipa/commit/4a08316) Add derive `IntoResponses` support (https://github.com/juhaku/utoipa/pull/433)
* [3d0ea69](https://github.com/juhaku/utoipa/commit/3d0ea69) Add `ToResponse` derive implementation (https://github.com/juhaku/utoipa/pull/416)
* [1af2443](https://github.com/juhaku/utoipa/commit/1af2443) Add support for repeated `schema(...)` definition (https://github.com/juhaku/utoipa/pull/410)
* [71f46ec](https://github.com/juhaku/utoipa/commit/71f46ec) Add external ref(...) attribute (https://github.com/juhaku/utoipa/pull/409)
* [abf4728](https://github.com/juhaku/utoipa/commit/abf4728) Add example attributes for request body (https://github.com/juhaku/utoipa/pull/406)
* [d4489d1](https://github.com/juhaku/utoipa/commit/d4489d1) Add auto detect application/octet-stream type (https://github.com/juhaku/utoipa/pull/405)
* [40b8bc0](https://github.com/juhaku/utoipa/commit/40b8bc0) Add support for chrono NaiveDate (https://github.com/juhaku/utoipa/pull/404)
* [8b541cf](https://github.com/juhaku/utoipa/commit/8b541cf) Add support for multiple examples in response (https://github.com/juhaku/utoipa/pull/403)
* [8e3bed2](https://github.com/juhaku/utoipa/commit/8e3bed2) Add Example type to OpenApi types (https://github.com/juhaku/utoipa/pull/402)
* [30a45d8](https://github.com/juhaku/utoipa/commit/30a45d8) Add derive info support for derive OpenApi (https://github.com/juhaku/utoipa/pull/400)
* [4d982c6](https://github.com/juhaku/utoipa/commit/4d982c6) Add `merge` functionality for `OpenApi` (https://github.com/juhaku/utoipa/pull/397)
* [7026f9e](https://github.com/juhaku/utoipa/commit/7026f9e) Add derive servers attribute for OpenApi (https://github.com/juhaku/utoipa/pull/395)
* [a4b1af0](https://github.com/juhaku/utoipa/commit/a4b1af0) Add support for unit sructs (https://github.com/juhaku/utoipa/pull/392)
* [bb1717b](https://github.com/juhaku/utoipa/commit/bb1717b) Add support for `schema_with` custon fn reference (https://github.com/juhaku/utoipa/pull/390)
* [391daef](https://github.com/juhaku/utoipa/commit/391daef) Add support for multiple serde definitions (https://github.com/juhaku/utoipa/pull/389)
* [0cf8eae](https://github.com/juhaku/utoipa/commit/0cf8eae) Add support for tuple Path parameters for axum (https://github.com/juhaku/utoipa/pull/388)
* [7661aab](https://github.com/juhaku/utoipa/commit/7661aab) Add derive validation attributes to `IntoParams` (https://github.com/juhaku/utoipa/pull/386)
* [23f517c](https://github.com/juhaku/utoipa/commit/23f517c) Add support for derive validation attributes (https://github.com/juhaku/utoipa/pull/385)
* [ae840e3](https://github.com/juhaku/utoipa/commit/ae840e3) Add support for missing attributes for validation (https://github.com/juhaku/utoipa/pull/379)@juhaku 
* [093014e](https://github.com/juhaku/utoipa/commit/093014e) Add support for multiple return types (https://github.com/juhaku/utoipa/pull/377)
* [63613c2](https://github.com/juhaku/utoipa/commit/63613c2) Add support for self referencing schema (https://github.com/juhaku/utoipa/pull/375)
* [483b778](https://github.com/juhaku/utoipa/commit/483b778) Add missing features to `IntoParams` (https://github.com/juhaku/utoipa/pull/374)

### Fixed

* [2e13074](https://github.com/juhaku/utoipa/commit/2e13074) Fix spelling (https://github.com/juhaku/utoipa/pull/450)
* [e50da56](https://github.com/juhaku/utoipa/commit/e50da56) Fix empty string path parameter name for Axum (https://github.com/juhaku/utoipa/pull/424)
* [b0f2eb1](https://github.com/juhaku/utoipa/commit/b0f2eb1) Fix typo in doc
* [db19971](https://github.com/juhaku/utoipa/commit/db19971) Fix make untagged enum object variants required (https://github.com/juhaku/utoipa/pull/407)
* [9639a06](https://github.com/juhaku/utoipa/commit/9639a06) Fix time-crate typo in schema format tokens (https://github.com/juhaku/utoipa/pull/401)
* [9a482c9](https://github.com/juhaku/utoipa/commit/9a482c9) Fix primitive type generic aliases (https://github.com/juhaku/utoipa/pull/393)
* [fdd244c](https://github.com/juhaku/utoipa/commit/fdd244c) Fix `TypeTree` for `slice` and `array` types (https://github.com/juhaku/utoipa/pull/387)
* [67b9585](https://github.com/juhaku/utoipa/commit/67b9585) Fix minum axum version info
* [a6f41cf](https://github.com/juhaku/utoipa/commit/a6f41cf) Fix broken swagger-ui on axum  (https://github.com/juhaku/utoipa/pull/370)

### Changed

* [d19aadf](https://github.com/juhaku/utoipa/commit/d19aadf) Refactor `ToReponse` trait (https://github.com/juhaku/utoipa/pull/460)
* [11288c3](https://github.com/juhaku/utoipa/commit/11288c3) Refactor to schema casting as (https://github.com/juhaku/utoipa/pull/459)
* [5b51eb4](https://github.com/juhaku/utoipa/commit/5b51eb4) Enhance generic aliases with lifetimes support (https://github.com/juhaku/utoipa/pull/458)
* [46fe673](https://github.com/juhaku/utoipa/commit/46fe673) Enhance path tuple argument support (https://github.com/juhaku/utoipa/pull/455)
* [13a3aae](https://github.com/juhaku/utoipa/commit/13a3aae) Update versions
* [50ab67f](https://github.com/juhaku/utoipa/commit/50ab67f) Update README
* [f37261e](https://github.com/juhaku/utoipa/commit/f37261e) Update docs
* [5a78fef](https://github.com/juhaku/utoipa/commit/5a78fef) Improve docs (https://github.com/juhaku/utoipa/pull/444)
* [084b2a1](https://github.com/juhaku/utoipa/commit/084b2a1) Enhance reponses derive support (https://github.com/juhaku/utoipa/pull/443)
* [284b01a](https://github.com/juhaku/utoipa/commit/284b01a) Document `preserve_order` feature flag (https://github.com/juhaku/utoipa/pull/437)
* [28e64ad](https://github.com/juhaku/utoipa/commit/28e64ad) Feat/serde enum representation (https://github.com/juhaku/utoipa/pull/414)
* [571fc10](https://github.com/juhaku/utoipa/commit/571fc10) Enhance `ToResponse` implementation (https://github.com/juhaku/utoipa/pull/419)
* [d8c5ec5](https://github.com/juhaku/utoipa/commit/d8c5ec5) Swagger UI url config (https://github.com/juhaku/utoipa/pull/418)
* [ddb9eb3](https://github.com/juhaku/utoipa/commit/ddb9eb3) Addresss clippy lint
* [6c7f6a2](https://github.com/juhaku/utoipa/commit/6c7f6a2) Improve documenetation
* [5b643af](https://github.com/juhaku/utoipa/commit/5b643af) Enhance repeated attributes support (https://github.com/juhaku/utoipa/pull/411)
* [7138fd9](https://github.com/juhaku/utoipa/commit/7138fd9) Make derive OpenApi server variable names LitStr
* [79ab858](https://github.com/juhaku/utoipa/commit/79ab858) Refactor `Type` to `TypeTree` (https://github.com/juhaku/utoipa/pull/408)
* [fed7237](https://github.com/juhaku/utoipa/commit/fed7237) Update `ToSchema` documentation
* [2679108](https://github.com/juhaku/utoipa/commit/2679108) feat: make schema_type and title pub on Object (https://github.com/juhaku/utoipa/pull/382)
* [daa9f43](https://github.com/juhaku/utoipa/commit/daa9f43) Use BTreeMap for responses of components to make it fixed order (https://github.com/juhaku/utoipa/pull/380)
* [772c089](https://github.com/juhaku/utoipa/commit/772c089) Chore make `serde_json` mandatory dependency (https://github.com/juhaku/utoipa/pull/378)
* [1df2773](https://github.com/juhaku/utoipa/commit/1df2773) Feature http status codes (https://github.com/juhaku/utoipa/pull/376)
* [436cccb](https://github.com/juhaku/utoipa/commit/436cccb) Refactor some path derive with `IntoParmas` tests
* [afa62d2](https://github.com/juhaku/utoipa/commit/afa62d2) Update utoipa-swagger-ui install example
* [65842b9](https://github.com/juhaku/utoipa/commit/65842b9) Chore refine description attribute (https://github.com/juhaku/utoipa/pull/373)
* [6971fb3](https://github.com/juhaku/utoipa/commit/6971fb3) Update swagger-ui dependencies versions
* [5b2c8ef](https://github.com/juhaku/utoipa/commit/5b2c8ef) Update utoipa-swagger-ui version
* [badb475](https://github.com/juhaku/utoipa/commit/badb475) cargo format
* [b558b36](https://github.com/juhaku/utoipa/commit/b558b36) Update to axum 0.6.0 (https://github.com/juhaku/utoipa/pull/369)

