//! The misuses the types refuse, each a fixture that must not compile.

// A loom run tests the models alone.
#![cfg(not(loom))]

#[cfg(test)]
mod tests {
    use trybuild::TestCases;

    #[test]
    fn each_misuse_fails_to_compile() {
        TestCases::new().compile_fail("tests/compile_fail/*.rs");
    }
}
