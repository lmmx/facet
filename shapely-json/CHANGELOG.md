# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [3.1.1](https://github.com/bearcove/shapely/compare/shapely-json-v3.1.0...shapely-json-v3.1.1) - 2025-04-05

### Fixed

- fix json tests
- fix miri/memory problems

### Other

- C demo
- shapedebug bye
- The split has gone well
- woo everything builds
- getting there
- good, good
- the rest is TODOs
- json tests almost passing
- quicksave
- brrrr
- set up json again
- restore json test a little?? not much
- Use spez-like ideas to set Debug if it's set on the type
- alright, will this work?
- Well the tests do pass
- don't compare strings
- Make CI pass
- json works I guess
- pv.shape eq is always true? woops
- bit of json as a treat
- introduce init_in_place_with_capacity
- whoa hey down to 111 errors
- Innards => Def
- Introduce poke

## [3.1.0](https://github.com/bearcove/shapely/compare/shapely-json-v3.0.0...shapely-json-v3.1.0) - 2025-03-31

### Added

- introduce NameOpts
- add support for numeric scalar types in from_json
- Implement parsing for various numeric scalars in JSON

### Other

- full docs baybee
- Fill in missing docs
- Improve naming
- errors--
- Fix all markdown links
- unfuck docs
- Yay for miri
- woops, edition 2024
- json roundtrip
- arrays.. work?
- commented out test with vec
- support tuples, booleans etc.
- Distinguish structs, tuples, and tuple structs
- clippy fixes
- pre-commit hook with cargo-fmt
- shill for namespace, closes #36
- specific toolchains, reformat code
- Improve debug output
- woo

## [2.0.0](https://github.com/bearcove/shapely/compare/shapely-json-v1.0.0...shapely-json-v2.0.0) - 2025-03-11

### Other

- Simplify Shape name function signature
- Change Shape.name from static str to NameFn
- Back to something working
- Tests pass again
- Add `addr` and `shape` methods to Partial and Slot
- Looking good!
- Clean up Partial API
- Shape up field API
- Stability notes
- introduce slot_by_index / slot_by_name
- link to 'free of syn' campaign
- Ensure no syn dependency (and badge about it), closes #9
- Introduce core crate
- Get rid of debug/display, closes #4
