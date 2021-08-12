# pinot

Fast, high-fidelity, zero-copy OpenType parser.

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
[![Apache licensed][apache-badge]][apache-url]

[crates-badge]: https://img.shields.io/crates/v/pinot.svg
[crates-url]: https://crates.io/crates/pinot
[docs-badge]: https://docs.rs/pinot/badge.svg
[docs-url]: https://docs.rs/pinot
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: LICENSE-MIT
[apache-badge]: https://img.shields.io/badge/license-Apache--2.0-blue.svg
[apache-url]: LICENSE-APACHE

This crate is a work in progress, but aims to parse OpenType fonts with a level
of detail that is amenable to modeling, analysis and transformation. The current 
focus is specifically on OpenType layout and the crate provides 
comprehensive coverage of that portion of the specification along with strong
support for variations and the core header tables. There are still a lot of
missing bits to cover the full specification.

The long term (and perhaps overly ambitious) goal is a community collaboration
on a set of crates that can do for font tooling what LLVM has done for compiler
tooling. Specifically, there is a desire to build a generic font model abstraction
(in some other crate) that is somewhat akin to LLVM IR and allows for analysis,
transformations and optimization. With importers from various formats 
(OpenType, UFO, Glyphs, ..) and exporters for others (OpenType, OpenType-Next?)
the idea is to support current font development pipelines while allowing
exploration for future progress.
