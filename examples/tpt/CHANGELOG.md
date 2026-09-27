# Changelog

All notable changes to this example are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `estimate` subcommand: neutral model JSON + CSV cost database to priced
  `.xlsx` estimate via the takeoff and estimating engines.
- `serve` subcommand (behind the `serve` feature): exposes takeoff, estimate,
  and CPM scheduling over a dependency-free HTTP interface with optional
  bearer-token auth.
