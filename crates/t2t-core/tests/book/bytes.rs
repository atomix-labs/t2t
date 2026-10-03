//! The zerocopy listing of the chapter Serialization.

#[test]
// ANCHOR: bytes
fn a_value_is_read_from_bytes_and_a_point_written_to_them() {
    use t2t_core::{Timed, Timestamp, UtcDateTime};
    use zerocopy::{FromBytes, IntoBytes};

    // A point is its count, native-endian, both ways.
    let instant = Timestamp::from_secs(1_700_000_000);
    let bytes = instant.as_bytes();
    assert_eq!(bytes, 1_700_000_000_000_000_000_i64.to_ne_bytes(), "the count's bytes");
    assert_eq!(Timestamp::read_from_bytes(bytes), Ok(instant), "and back");

    // A stamped value is read from bytes: the stamp's, then the value's.
    let mut record = [0_u8; 16];
    record[..8].copy_from_slice(bytes);
    record[8..].copy_from_slice(&101_u64.to_ne_bytes());
    let quote = Timed::<u64>::read_from_bytes(&record).expect("sixteen bytes");
    assert_eq!((quote.stamp, quote.value), (instant, 101), "a stamped word");

    // Any bytes make a date and time, so one read from them is checked before it is used.
    let date_time = UtcDateTime::read_from_bytes(&[0xFF; 16]).expect("sixteen bytes");
    assert!(!date_time.is_valid(), "month 255 is no month");
    assert_eq!(date_time.to_timestamp(), None, "so it names no instant");
}
// ANCHOR_END: bytes
