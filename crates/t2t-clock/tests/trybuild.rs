//! The misuses the types refuse, each a fixture that must not compile.

// A loom run tests the models alone.
#![cfg(not(loom))]

#[cfg(test)]
mod tests {
    #[test]
    fn each_misuse_fails_to_compile() {
        trybuild::TestCases::new().compile_fail("tests/compile_fail/*.rs");
    }
}
