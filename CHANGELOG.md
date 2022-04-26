# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased] - ReleaseDate
### Fixed
- Update internal `FancyGuard` type to use atomics rather than `Cell` to ensure
  that futures using spandoc remain `Send`

## [0.2.1] - 2020-11-11
### Fixed
- Fixed compiler error when using `#[spandoc]` on functions containing an
  empty block

<!-- next-url -->
[Unreleased]: https://github.com/yaahc/spandoc/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/yaahc/spandoc/releases/tag/v0.2.1
