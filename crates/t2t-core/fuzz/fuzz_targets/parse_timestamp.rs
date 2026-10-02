//! `Timestamp::from_str` over any UTF-8, which it refuses or reads and never panics on; whatever it
//! reads, its spelling reads back as the same instant.

#![no_main]

use libfuzzer_sys::{Corpus, fuzz_target};
use t2t_core::Timestamp;

fuzz_target!(|data: &[u8]| -> Corpus {
    let Ok(text) = core::str::from_utf8(data) else {
        return Corpus::Reject;
    };
    if let Ok(instant) = text.parse::<Timestamp>() {
        assert_eq!(instant.to_string().parse(), Ok(instant), "the spelling of {text:?} reads back");
    }
    Corpus::Keep
});
