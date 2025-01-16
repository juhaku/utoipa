# Changelog - utoipa-rapidoc

## 6.0.0 - Thu 16 2025

### Changed

* Re-release 5.0.1 since axum upgrade is a breaking change (https://github.com/juhaku/utoipa/pull/1295)

## 5.0.1 - Jan 6 2025

### Changed

* Update axum to v0.8 (https://github.com/juhaku/utoipa/pull/1269)

## 5.0.0 - Oct 14 2024

### Added

* Add macros feature flag (https://github.com/juhaku/utoipa/pull/1015)
* Add nest `OpenApi` support (https://github.com/juhaku/utoipa/pull/930)

### Fixed

* Fix testing without explicit features (https://github.com/juhaku/utoipa/pull/1041)
* Fix building utoipa-rapidoc & utoipa-scalar (https://github.com/juhaku/utoipa/pull/1039)
* Fix RapiDoc with empty URL panic on axum (https://github.com/juhaku/utoipa/pull/997)
* Fix samples in documentation when with_openapi is used (https://github.com/juhaku/utoipa/pull/988)
* Fix openapi serialized twice when served with Rocket (https://github.com/juhaku/utoipa/pull/987)

### Changed

* Remove commit commit id from changelogs (https://github.com/juhaku/utoipa/pull/1077)
* Update to rc version
* Disable unused default features of rust_decimal (https://github.com/juhaku/utoipa/pull/1029)
* Update next beta versions
* Chore update docs and relax `url` version (https://github.com/juhaku/utoipa/pull/1001)
* Bump up versions (https://github.com/juhaku/utoipa/pull/998)
* Update versions
* Update min Rust version and utoipa-swagger-ui version

## 4.0.0 - May 5 2024

### Added

* Add `default-features = false` to the optional axum dependency to avoid pulling in tokio in non-tokio environments (https://github.com/juhaku/utoipa/pull/874)

### Fixed

* Fix spelling (https://github.com/juhaku/utoipa/pull/846)

### Changed

* Update docs and next versions
* Refactor RapiDoc to take `Cow<'static, str>` instead of borrowed `str` (https://github.com/juhaku/utoipa/pull/867)

## 3.0.0 - Jan 9 2024

### Fixed

* fix: fix typo (https://github.com/juhaku/utoipa/pull/822)

### Changed

* Update next versions
* Update Rocket v0.5 (https://github.com/juhaku/utoipa/pull/825)
* Misc document improvements (https://github.com/juhaku/utoipa/pull/814)

## 2.0.0 - Nov 28 2023

### Changed

* Update next versions
* Axum 0.7 bindings (https://github.com/juhaku/utoipa/pull/807)

## 1.0.0 - Oct 7 2023

### Changed

* Update utoipa versions **min. utoipa: `4`**

## 0.1.0 - Aug 8 2023

### Added

* Add rapidoc support (https://github.com/juhaku/utoipa/pull/723)

