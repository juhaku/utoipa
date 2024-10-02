# Changelog - utoipa-rapidoc

## Unreleased

### Added

* [11c909b](https://github.com/juhaku/utoipa/commit/11c909b) Add macros feature flag (https://github.com/juhaku/utoipa/pull/1015)
* [97bc507](https://github.com/juhaku/utoipa/commit/97bc507) Add nest `OpenApi` support (https://github.com/juhaku/utoipa/pull/930)

### Fixed

* [2d81c9b](https://github.com/juhaku/utoipa/commit/2d81c9b) Fix testing without explicit features (https://github.com/juhaku/utoipa/pull/1041)
* [4422cee](https://github.com/juhaku/utoipa/commit/4422cee) Fix building utoipa-rapidoc & utoipa-scalar (https://github.com/juhaku/utoipa/pull/1039)
* [2c76479](https://github.com/juhaku/utoipa/commit/2c76479) Fix RapiDoc with empty URL panic on axum (https://github.com/juhaku/utoipa/pull/997)
* [c774742](https://github.com/juhaku/utoipa/commit/c774742) Fix samples in documentation when with_openapi is used (https://github.com/juhaku/utoipa/pull/988)
* [2406c75](https://github.com/juhaku/utoipa/commit/2406c75) Fix openapi serialized twice when served with Rocket (https://github.com/juhaku/utoipa/pull/987)

### Changed

* [18d004a](https://github.com/juhaku/utoipa/commit/18d004a) Disable unused default features of rust_decimal (https://github.com/juhaku/utoipa/pull/1029)
* [57ba3ba](https://github.com/juhaku/utoipa/commit/57ba3ba) Update next beta versions
* [1af4ad4](https://github.com/juhaku/utoipa/commit/1af4ad4) Chore update docs and relax `url` version (https://github.com/juhaku/utoipa/pull/1001)
* [89c288b](https://github.com/juhaku/utoipa/commit/89c288b) Bump up versions (https://github.com/juhaku/utoipa/pull/998)
* [d020f92](https://github.com/juhaku/utoipa/commit/d020f92) Update versions
* [164c161](https://github.com/juhaku/utoipa/commit/164c161) Update min Rust version and utoipa-swagger-ui version

## 4.0.0 - May 5 2024

### Added

* [4b32ba9](https://github.com/juhaku/utoipa/commit/4b32ba9) Add `default-features = false` to the optional axum dependency to avoid pulling in tokio in non-tokio environments (https://github.com/juhaku/utoipa/pull/874)

### Fixed

* [8639185](https://github.com/juhaku/utoipa/commit/8639185) Fix spelling (https://github.com/juhaku/utoipa/pull/846)

### Changed

* [c907d43](https://github.com/juhaku/utoipa/commit/c907d43) Update docs and next versions
* [5c6b0e2](https://github.com/juhaku/utoipa/commit/5c6b0e2) Refactor RapiDoc to take `Cow<'static, str>` instead of borrowed `str` (https://github.com/juhaku/utoipa/pull/867)

## 3.0.0 - Jan 9 2024

### Fixed

* [a968ced](https://github.com/juhaku/utoipa/commit/a968ced) fix: fix typo (https://github.com/juhaku/utoipa/pull/822)

### Changed

* [f7cae03](https://github.com/juhaku/utoipa/commit/f7cae03) Update next versions
* [d437919](https://github.com/juhaku/utoipa/commit/d437919) Update Rocket v0.5 (https://github.com/juhaku/utoipa/pull/825)
* [7e49344](https://github.com/juhaku/utoipa/commit/7e49344) Misc document improvements (https://github.com/juhaku/utoipa/pull/814)

## 2.0.0 - Nov 28 2023

### Changed

* [35f32b1](https://github.com/juhaku/utoipa/commit/35f32b1) Update next versions
* [93dfaf1](https://github.com/juhaku/utoipa/commit/93dfaf1) Axum 0.7 bindings (https://github.com/juhaku/utoipa/pull/807)

## 1.0.0 - Oct 7 2023

### Changed

* [50db1b0](https://github.com/juhaku/utoipa/commit/50db1b0) Update utoipa versions **min. utoipa: `4`**

## 0.1.0 - Aug 8 2023

### Added

* [56b6326](https://github.com/juhaku/utoipa/commit/56b6326) Add rapidoc support (https://github.com/juhaku/utoipa/pull/723)

