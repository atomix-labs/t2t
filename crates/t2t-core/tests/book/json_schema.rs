//! The JSON Schema listing of the chapter Serialization.

#[test]
// ANCHOR: schema
fn an_instant_and_a_span_have_the_json_schemas_of_their_spellings() {
    use schemars::schema_for;
    use t2t_core::{Timedelta, Timestamp};

    let instant = schema_for!(Timestamp);
    let field = |name: &str| instant.get(name).and_then(|value| value.as_str());
    assert_eq!(field("type"), Some("string"), "an instant is a string");
    assert_eq!(field("format"), Some("date-time"), "in RFC 3339's format");

    let span = schema_for!(Timedelta);
    let pattern = span.get("pattern").and_then(|value| value.as_str());
    let expected = "^(0|-?(?=[0-9])([0-9]+d)?([0-9]+h)?([0-9]+m)?([0-9]+s)?([0-9]+ms)?([0-9]+us)?([0-9]+ns)?)$";
    assert_eq!(pattern, Some(expected), "a span's pattern takes what its parser reads");
}
// ANCHOR_END: schema
