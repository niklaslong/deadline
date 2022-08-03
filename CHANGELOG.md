# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0]

### Changed

- The output of `deadline` on panic now doesn't include `move ||` and only contains the closure body.

## [0.1.1]

### Fixed

- Wake the task and check condition with each poll in the `Future` implementation so that `deadline` returns as soon as the closure returns `true`.


[unreleased]: https://github.com/niklaslong/deadline/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/niklaslong/deadline/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/niklaslong/deadline/compare/v0.1.0...v0.1.1

