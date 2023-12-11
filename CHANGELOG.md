# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.7](https://github.com/lsunsi/lnurlkit/compare/v0.1.6...v0.1.7) - 2023-12-11

### Other
- leverage serde for serding the query strings
- cave in and replace miniserde by serde
- split client/server structs inside core

## [0.1.6](https://github.com/lsunsi/lnurlkit/compare/v0.1.5...v0.1.6) - 2023-12-08

### Added
- add intermediary callback request struct to support more use cases

### Other
- use request types on server callbacks

## [0.1.5](https://github.com/lsunsi/lnurlkit/compare/v0.1.4...v0.1.5) - 2023-12-06

### Added
- *(pay)* add hack that saves metadata raw on pay query

### Fixed
- fix repository link on cargo toml

### Other
- *(pay)* make comment size optional on query construction

## [0.1.4](https://github.com/lsunsi/lnurlkit/compare/v0.1.3...v0.1.4) - 2023-12-06

### Added
- *(pay)* add support for lud16 (pay to identifier)

### Other
- add test for lud01 (even thought it's redundant)
- add test with real world payreq parsing

## [0.1.3](https://github.com/lsunsi/lnurlkit/compare/v0.1.2...v0.1.3) - 2023-12-05

### Added
- *(channel)* add action to channel server callback
- *(withdraw)* add k1 to withdraw server callback
- *(channel)* add client/server and a test for withdraw (lud02)
- *(withdraw)* add client/server and a test for withdraw (lud03)

### Fixed
- change bitor for logical one (typo)

### Other
- cover serializations with unit tests
- remove nedless clone on urls serialization
- add re-exports top level for ease of use
- rename, reorganize, simplify, evolve
- rename withdrawal to withdraw (so confusing)

## [0.1.2](https://github.com/lsunsi/lnurlkit/compare/v0.1.1...v0.1.2) - 2023-12-04

### Fixed
- *(doc)* add features and nightly stuff to doc

## [0.1.1](https://github.com/lsunsi/lnurlkit/compare/v0.1.0...v0.1.1) - 2023-12-04

- Major reorganization
- More tests, features, structs. The works
