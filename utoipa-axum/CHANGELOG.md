# Changelog - utoipa-axum

## 0.1.3 - Dec 19 2024

### Changed

* Allow trailing comma in `routes!()` macro (https://github.com/juhaku/utoipa/pull/1238)

### Fixed

* Fix axum path nesting (https://github.com/juhaku/utoipa/pull/1231)
* Fix diverging axum route and openapi spec (https://github.com/juhaku/utoipa/pull/1199)

## 0.1.2 - Oct 29 2024

### Changed

* Merge paths if they already are added to the paths map in utoipa-axum (https://github.com/juhaku/utoipa/pull/1171)

## 0.1.1 - Oct 16 2024

### Changed

* Use OpenApiRouter::default for empty OpenApi (https://github.com/juhaku/utoipa/pull/1133)

## 0.1.0 - Oct 14 2024

### Added

* Add auto collect schemas for utoipa-axum (https://github.com/juhaku/utoipa/pull/1072)
* Add typos to CI (https://github.com/juhaku/utoipa/pull/1036)
* Add paths support for routes! macro (https://github.com/juhaku/utoipa/pull/1023)
* Add macros feature flag (https://github.com/juhaku/utoipa/pull/1015)
* Add `utoipa-axum` binding example and update docs (https://github.com/juhaku/utoipa/pull/1007)
* Add support to define multiple operation methods (https://github.com/juhaku/utoipa/pull/1006)
* Add utoipa axum bindings (https://github.com/juhaku/utoipa/pull/1004)

### Fixed

* Fix `routes!` macro call (https://github.com/juhaku/utoipa/pull/1108)
* Fix testing without explicit features (https://github.com/juhaku/utoipa/pull/1041)
* Fix building utoipa-axum & utoipa-swagger-ui (https://github.com/juhaku/utoipa/pull/1038)
* Fix utoipa-axum project description
* Fix some typos

### Changed

* Fix typos
* Remove commit commit id from changelogs (https://github.com/juhaku/utoipa/pull/1077)
* Update to rc version
* Chore change the operations implementation. (https://github.com/juhaku/utoipa/pull/1026)
* Update utoipa-axum version
* Enhance `utoipa-axum` bindings (https://github.com/juhaku/utoipa/pull/1017)
* Update next beta versions

