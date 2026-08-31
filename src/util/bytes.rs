//! Big/little-endian packing of a 4-byte slice to/from an `i32`.
//!
//! Ports `ra.common.BytesUtil`. The Java version works with `int`; we keep the
//! signed 32-bit type for parity but the conversions are just the standard
//! library's byte-order helpers.

/// Interpret the first four bytes of `b` as a big-endian `i32`.
///
/// # Panics
/// Panics if `b` has fewer than 4 bytes, matching the Java `ArrayIndexOutOfBounds`.
pub fn pack_big_endian(b: &[u8]) -> i32 {
    i32::from_be_bytes([b[0], b[1], b[2], b[3]])
}

/// Encode `x` as four big-endian bytes.
pub fn unpack_big_endian(x: i32) -> [u8; 4] {
    x.to_be_bytes()
}

/// Interpret the first four bytes of `b` as a little-endian `i32`.
///
/// # Panics
/// Panics if `b` has fewer than 4 bytes.
pub fn pack_little_endian(b: &[u8]) -> i32 {
    i32::from_le_bytes([b[0], b[1], b[2], b[3]])
}

/// Encode `x` as four little-endian bytes.
pub fn unpack_little_endian(x: i32) -> [u8; 4] {
    x.to_le_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        for v in [0i32, 1, -1, 42, i32::MIN, i32::MAX, 0x0A0B_0C0D] {
            assert_eq!(pack_big_endian(&unpack_big_endian(v)), v);
            assert_eq!(pack_little_endian(&unpack_little_endian(v)), v);
        }
    }

    #[test]
    fn endianness() {
        assert_eq!(unpack_big_endian(0x0102_0304), [1, 2, 3, 4]);
        assert_eq!(unpack_little_endian(0x0102_0304), [4, 3, 2, 1]);
    }
}
