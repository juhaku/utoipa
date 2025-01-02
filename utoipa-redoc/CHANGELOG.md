# Changelog - utoipa-redoc

## Unreleased

### Changed

* Update axum to v0.8 (https://github.com/juhaku/utoipa/pull/1269)

## 5.0.0 - Oct 14 2024

### Added

* Add macros feature flag (https://github.com/juhaku/utoipa/pull/1015)
* Add nest `OpenApi` support (https://github.com/juhaku/utoipa/pull/930)

### Fixed

* Fix testing without explicit features (https://github.com/juhaku/utoipa/pull/1041)
* Fix building utoipa-rapidoc & utoipa-scalar (https://github.com/juhaku/utoipa/pull/1039)

### Changed

* Remove commit commit id from changelogs (https://github.com/juhaku/utoipa/pull/1077)
* Update to rc version
* Update next beta versions
* Chore fix clippy lint (https://github.com/juhaku/utoipa/pull/1010)
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
* Refactor `ReDoc` to take `Cow<'static, str>` instead of borrowed `str` (https://github.com/juhaku/utoipa/pull/869)

## 3.0.0 - Jan 9 2024

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

* Update next versions **min. utoipa: `4`**
* Axum 0.7 bindings (https://github.com/juhaku/utoipa/pull/807)

## 0.1.0 - Aug 6 2023

### Added

* Add redoc support for utoipa. (https://github.com/juhaku/utoipa/pull/720)

### Changed

* Update README.md docs

