# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **Breaking:** `deadline!` now takes a boolean expression instead of a closure. Migration is mechanical — delete `move ||` from the second argument:

  ```rust
  // before
  deadline!(Duration::from_millis(10), move || x.load(Ordering::Relaxed) == 42);
  // after
  deadline!(Duration::from_millis(10), x.load(Ordering::Relaxed) == 42);
  ```

- The internal condition helper now takes `FnMut` (was `Fn + 'static`), so conditions can mutate captured state and borrow non-`'static` data.
- Polling loop rewritten as `while !cond { yield_now().await }`. Same non-blocking, minimum-latency behavior as before, expressed more directly.
- Update `tokio` to `1.32` and enable the `rt` feature (required for `yield_now`).
- Minor cosmetic refactors.

## [0.2.0]

### Changed

- The output of `deadline` on panic now doesn't include `move ||` and only contains the closure body.

## [0.1.1]

### Fixed

- Wake the task and check condition with each poll in the `Future` implementation so that `deadline` returns as soon as the closure returns `true`.


[unreleased]: https://github.com/niklaslong/deadline/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/niklaslong/deadline/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/niklaslong/deadline/compare/v0.1.0...v0.1.1

