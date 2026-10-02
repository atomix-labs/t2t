//! `Timedelta::from_str` over any UTF-8, which it refuses or reads and never panics on; whatever it
//! reads, its spelling reads back as the same span.

#![no_main]

use libfuzzer_sys::{Corpus, fuzz_target};
use t2t_core::Timedelta;

fuzz_target!(|data: &[u8]| -> Corpus {
    let Ok(text) = core::str::from_utf8(data) else {
        return Corpus::Reject;
    };
    if let Ok(span) = text.parse::<Timedelta>() {
        assert_eq!(span.to_string().parse(), Ok(span), "the spelling of {text:?} reads back");
    }
    Corpus::Keep
});
