# Changelog - utoipa-swagger-ui

## Unreleased

### Added

* [9d778b0](https://github.com/juhaku/utoipa/commit/9d778b0) Add typos to CI (https://github.com/juhaku/utoipa/pull/1036)
* [11c909b](https://github.com/juhaku/utoipa/commit/11c909b) Add macros feature flag (https://github.com/juhaku/utoipa/pull/1015)

### Fixed

* [2d81c9b](https://github.com/juhaku/utoipa/commit/2d81c9b) Fix testing without explicit features (https://github.com/juhaku/utoipa/pull/1041)
* [fcdb5db](https://github.com/juhaku/utoipa/commit/fcdb5db) Fix building utoipa-axum & utoipa-swagger-ui (https://github.com/juhaku/utoipa/pull/1038)
* [a5695f0](https://github.com/juhaku/utoipa/commit/a5695f0) Fix utoipa-swagger-ui-vendored crates link

### Changed

* [2611200](https://github.com/juhaku/utoipa/commit/2611200) Update to rc
* [78656b6](https://github.com/juhaku/utoipa/commit/78656b6) Always use system `curl` by default (https://github.com/juhaku/utoipa/pull/1045)
* [a985d8c](https://github.com/juhaku/utoipa/commit/a985d8c) Remove Redirect Causing Invalid URIs for Swagger UIs Server on / (https://github.com/juhaku/utoipa/pull/1043)
* [c36f877](https://github.com/juhaku/utoipa/commit/c36f877) Use fs::read to overwrite swagger UI contents (https://github.com/juhaku/utoipa/pull/1022)
* [57ba3ba](https://github.com/juhaku/utoipa/commit/57ba3ba) Update next beta versions
* [1af4ad4](https://github.com/juhaku/utoipa/commit/1af4ad4) Chore update docs and relax `url` version (https://github.com/juhaku/utoipa/pull/1001)
* [89c288b](https://github.com/juhaku/utoipa/commit/89c288b) Bump up versions (https://github.com/juhaku/utoipa/pull/998)
* [d020f92](https://github.com/juhaku/utoipa/commit/d020f92) Update versions
* [68172bf](https://github.com/juhaku/utoipa/commit/68172bf) Update utoipa-swagger-ui vendored dependency

## 7.1.0 - May 22 2024

### Added

* [c63407d](https://github.com/juhaku/utoipa/commit/c63407d) Add vendored Swagger UI for utoipa (https://github.com/juhaku/utoipa/pull/941)

### Changed

* [68172bf](https://github.com/juhaku/utoipa/commit/68172bf) Update utoipa-swagger-ui vendored dependency
* [91a98ef](https://github.com/juhaku/utoipa/commit/91a98ef) Update utoipa-swagger-ui-vendored version
* [2ddc279](https://github.com/juhaku/utoipa/commit/2ddc279) Include res for crates
* [164c161](https://github.com/juhaku/utoipa/commit/164c161) Update min Rust version and utoipa-swagger-ui version

## 7.0.3 - May 22 2024

### Fixed

* [360e23e](https://github.com/juhaku/utoipa/commit/360e23e) Fix docs.rs build

## 7.0.2 - May 21 2024

### Added

* [139c035](https://github.com/juhaku/utoipa/commit/139c035) Add missing windows check to utoipa-swagger-ui build.rs
* [97bc507](https://github.com/juhaku/utoipa/commit/97bc507) Add nest `OpenApi` support (https://github.com/juhaku/utoipa/pull/930)

### Changed

* [72218c9](https://github.com/juhaku/utoipa/commit/72218c9) Update `utoipa-swagger-ui` versions (https://github.com/juhaku/utoipa/pull/938)
* [ffcd202](https://github.com/juhaku/utoipa/commit/ffcd202) Enhance utoipa-swagger-ui build (https://github.com/juhaku/utoipa/pull/936)
* [335407b](https://github.com/juhaku/utoipa/commit/335407b) Use CARGO_HTTP_CAINFO CA file in build script if present (https://github.com/juhaku/utoipa/pull/935)
* [19ff135](https://github.com/juhaku/utoipa/commit/19ff135) Improve file:// url parsing (https://github.com/juhaku/utoipa/pull/925)
* [272ceb8](https://github.com/juhaku/utoipa/commit/272ceb8) Make SWAGGER_UI_DOWNLOAD_URL support file:// urls (https://github.com/juhaku/utoipa/pull/923)

## 7.0.1 - May 7 2024

### Changed

* [8594735](https://github.com/juhaku/utoipa/commit/8594735) Make reqwest to use rustls instead of openssl (https://github.com/juhaku/utoipa/pull/912)

## 7.0.0 - May 5 2024

### Added

* [4b32ba9](https://github.com/juhaku/utoipa/commit/4b32ba9) Add `default-features = false` to the optional axum dependency to avoid pulling in tokio in non-tokio environments (https://github.com/juhaku/utoipa/pull/874)

### Fixed

* [8639185](https://github.com/juhaku/utoipa/commit/8639185) Fix spelling (https://github.com/juhaku/utoipa/pull/846)

### Changed

* [19fb23c](https://github.com/juhaku/utoipa/commit/19fb23c) Seems like the zip_next is nowadays just zip again
* [c907d43](https://github.com/juhaku/utoipa/commit/c907d43) Update docs and next versions
* [926a5e8](https://github.com/juhaku/utoipa/commit/926a5e8) Update default Swagger UI version (https://github.com/juhaku/utoipa/pull/905)
* [4d798bc](https://github.com/juhaku/utoipa/commit/4d798bc) Replace `zip` with `zip_next` (https://github.com/juhaku/utoipa/pull/889)
* [776f753](https://github.com/juhaku/utoipa/commit/776f753) **`breaking`** Add flex to `utoipa-swagger-ui` build (https://github.com/juhaku/utoipa/pull/845)

## 6.0.0 - Jan 6 2024

### Changed

* [f7cae03](https://github.com/juhaku/utoipa/commit/f7cae03) Update next versions
* [fe229e2](https://github.com/juhaku/utoipa/commit/fe229e2) Allowing utoipa/utoipa-swagger-ui successfully build on Windows and made path proc macro attribute more permissive (https://github.com/juhaku/utoipa/pull/830)
* [d437919](https://github.com/juhaku/utoipa/commit/d437919) Update Rocket v0.5 (https://github.com/juhaku/utoipa/pull/825)
* [3e5a635](https://github.com/juhaku/utoipa/commit/3e5a635) Generate embed code instead of using interpolation (https://github.com/juhaku/utoipa/pull/828)
* [7e49344](https://github.com/juhaku/utoipa/commit/7e49344) Misc document improvements (https://github.com/juhaku/utoipa/pull/814)

## 5.0.0 - Nov 28 2023

### Changed

* [35f32b1](https://github.com/juhaku/utoipa/commit/35f32b1) Update next versions
* [93dfaf1](https://github.com/juhaku/utoipa/commit/93dfaf1) Axum 0.7 bindings (https://github.com/juhaku/utoipa/pull/807)

## 4.0.0 - Oct 7 2023

### Changed

* [50db1b0](https://github.com/juhaku/utoipa/commit/50db1b0) Update utoipa versions **min. utoipa `4`**

## 3.1.6 - Oct 7 2023

### Added

* [56b6326](https://github.com/juhaku/utoipa/commit/56b6326) Add rapidoc support (https://github.com/juhaku/utoipa/pull/723)

### Fixed

* [4e10648](https://github.com/juhaku/utoipa/commit/4e10648) Fix: panic on missing trailing / in rocket environment (https://github.com/juhaku/utoipa/pull/645) (#757)

### Changed

* [15053c5](https://github.com/juhaku/utoipa/commit/15053c5) Update next versions and dependencies

## 3.1.5 - Aug 6 2023

### Changed

* [30551f6](https://github.com/juhaku/utoipa/commit/30551f6) Update utoipa-swagger-ui version
* [f9d26f6](https://github.com/juhaku/utoipa/commit/f9d26f6) Update Swagger UI to 5.3.1
* [e5f7f70](https://github.com/juhaku/utoipa/commit/e5f7f70) Update README.md docs
