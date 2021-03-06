//! Functions that work with elliptic curve keys and signatures.

use libc::{c_int, c_uchar, size_t};
use std::ptr;

// OpenSSL's numeric code for the particular elliptic curve that Bitcoin uses.
#[allow(non_upper_case_globals)]
static NID_secp256k1: int = 714;

#[allow(non_camel_case_types)]
#[repr(C)]
struct EC_GROUP;

#[allow(non_camel_case_types)]
#[repr(C)]
struct EC_POINT;

#[allow(non_camel_case_types)]
#[repr(C)]
struct BIGNUM;

#[allow(non_camel_case_types)]
#[repr(C)]
struct BN_CTX;

#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[repr(C)]
enum point_conversion_form_t {
    POINT_CONVERSION_COMPRESSED = 2,
    POINT_CONVERSION_UNCOMPRESSED = 4,
    POINT_CONVERSION_HYBRID = 6
}

#[link(name = "crypto")]
extern {
    fn EC_POINT_new(group: *const EC_GROUP) -> *mut EC_POINT;
    fn EC_POINT_free(point: *mut EC_POINT);
    fn EC_POINT_mul(group: *const EC_GROUP,
                    r: *mut EC_POINT,
                    n: *const BIGNUM,
                    q: *const EC_POINT,
                    m: *const BIGNUM,
                    ctx: *mut BN_CTX) -> c_int;
    fn EC_POINT_point2oct(group: *const EC_GROUP,
                          p: *const EC_POINT,
                          form: point_conversion_form_t,
                          buf: *mut c_uchar,
                          len: size_t,
                          ctx: *mut BN_CTX) -> size_t;

    fn EC_GROUP_new_by_curve_name(nid: c_int) -> *mut EC_GROUP;

    fn BN_new() -> *mut BIGNUM;
    fn BN_free(a: *mut BIGNUM);
    fn BN_bin2bn(s: *const c_uchar, len: c_int, ret: *mut BIGNUM) -> *mut BIGNUM;

    fn BN_CTX_new() -> *mut BN_CTX;
    fn BN_CTX_free(c: *mut BN_CTX);
}

/// Takes a 32-byte Bitcoin private key, and derives the 65-byte uncompressed
/// public key from it. Assumes the private key is valid, i.e. is 32 bytes long
/// and falls within the range defined in `src/protocol/private_key.rs`.
pub fn derive_public_key(private_key: &[u8]) -> Vec<u8> {
    assert!(private_key.len() == 32u);
    unsafe {
        // Convert private key to OpenSSL's bignum type.
        let priv_key = BN_bin2bn(private_key.as_ptr(), private_key.len() as c_int, BN_new());

        // Initialize the secp256k1 elliptic curve used by Bitcoin.
        let curve = EC_GROUP_new_by_curve_name(NID_secp256k1 as c_int) as *const EC_GROUP;

        // Init a context for doing bignum stuff.
        let ctx = BN_CTX_new();

        // Use elliptic curve point multiplication to derive the public key.
        let pub_key = EC_POINT_new(curve);
        EC_POINT_mul(curve, pub_key, priv_key as *const BIGNUM, ptr::null(), ptr::null(), ctx);

        // Convert public key point to the actual key, in uncompressed format.
        let mut result = Vec::from_elem(65, 0u8);
        EC_POINT_point2oct(curve, pub_key as *const EC_POINT, point_conversion_form_t::POINT_CONVERSION_UNCOMPRESSED, result.as_mut_ptr(), 65, ctx);
        *result.index_mut(&0) = 0x04;

        // Free the allocated resources.
        BN_CTX_free(ctx);
        EC_POINT_free(pub_key);
        BN_free(priv_key);

        result
    }
}

#[cfg(test)]
mod tests {
    use super::derive_public_key;

    #[test]
    fn test_derive_public_key() {
        let private_key: &[u8] =
            &[0xf7,0x47,0x65,0x32,0xfe,0x57,0x53,0xeb,0xcb,0xea,0x26,0xfe,0x02,0xff,0xf1,0x8b,
              0xf0,0x15,0x54,0x6f,0x85,0xca,0xf7,0x8a,0xc8,0xd5,0x99,0x54,0x7f,0x7d,0x3a,0xac];
        let actual_public_key: &[u8] =
            &[0x04,0xd6,0x63,0x0e,0x2f,0x4f,0xb6,0xd6,0x2e,0xf5,0xbc,0x5b,0xe8,0x50,0x08,0x36,0x25,
                   0xc9,0xb5,0x84,0xf6,0x61,0xaa,0xf7,0x72,0x3b,0xd8,0x39,0x4d,0xb5,0xf6,0x14,0x49,
                   0x41,0xf6,0xb5,0xf8,0x34,0x42,0xd9,0x39,0x1d,0x77,0x4c,0x7d,0x7f,0x26,0x2c,0xe6,
                   0xc5,0x53,0x80,0xe0,0x96,0x44,0x23,0x05,0x36,0x72,0x70,0xb0,0x4a,0xca,0x6b,0x75];
        let derived_public_key = derive_public_key(private_key);

        assert_eq!(derived_public_key.as_slice(), actual_public_key);
    }
}

