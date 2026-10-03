//! The book's listings that need a crate the facade does not depend on, such as `serde_json` or
//! `chrono`: each a test, so a page shows code that compiles and runs.
//!
//! A page includes a listing by the anchor around it. A chapter's listings are in the module named
//! for it, and those of Serialization that need `zerocopy` or `schemars` in `bytes` and
//! `json_schema`; each module is built where the features it reads are on.

#[cfg(test)]
#[cfg(feature = "zerocopy")]
mod bytes;
#[cfg(test)]
#[cfg(feature = "schemars")]
mod json_schema;
#[cfg(test)]
#[cfg(feature = "serde")]
mod serialization;
#[cfg(test)]
mod working_with_other_crates;
