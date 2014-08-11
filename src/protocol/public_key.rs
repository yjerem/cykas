use util;
use protocol::private_key::PrivateKey;
use protocol::address::Address;

// TODO: support compressed public keys?
static LENGTH: uint = 65u;

// A Bitcoin public key is 65 bytes, consisting of a 0x04 byte (indicating it
// is in uncompressed format), a 32-byte X coordinate, and a 32-byte Y
// coordinate.
pub struct PublicKey {
    data: Vec<u8>
}

impl PublicKey {
    // Creates a PublicKey from the given raw data. Returns None if the data is
    // invalid.
    pub fn new(data: &[u8]) -> Option<PublicKey> {
        let key = PublicKey { data: data.to_vec() };
        if key.is_valid() {
            Some(key)
        } else {
            None
        }
    }

    // Creates a PublicKey from a PrivateKey.
    pub fn from_private_key(private_key: &PrivateKey) -> PublicKey {
        PublicKey { data: util::ecdsa::derive_public_key(private_key.get_data()) }
    }

    // Checks if the given public key is valid.
    fn is_valid(&self) -> bool {
        self.data.len() == LENGTH &&
        self.data[0] == 0x04
    }

    // Gets the raw data as a slice of bytes.
    pub fn get_data(&self) -> &[u8] {
        self.data.as_slice()
    }

    // Derives the address from the public key.
    pub fn to_address(&self) -> Address {
        Address::from_public_key(self)
    }
}

#[cfg(test)]
mod tests {
    use serialize::hex::FromHex;

    use util;
    use protocol::private_key::PrivateKey;

    use super::PublicKey;

    #[test]
    fn test_new() {
        let data = "04904B5CC692ECED64B2C04821F6A2D795BC3BC02F46165F95B817AF8A7810830D\
                      5BD4895315905B429EAEA4424908B3289668E46A2D1E451B2C9365120EB6D565";
        let data = data.from_hex().unwrap();
        let public_key = PublicKey::new(data.as_slice());
        assert!(public_key.is_some());
        assert_eq!(public_key.unwrap().get_data(), data.as_slice());
    }

    #[test]
    fn test_new_invalid_length() {
        let data = "04904B5CC692ECED64B2C04821F6A2D795BC3BC02F46165F95B817AF8A78108301";
        let data = data.from_hex().unwrap();
        let public_key = PublicKey::new(data.as_slice());
        assert!(public_key.is_none());
    }

    #[test]
    fn test_new_invalid_initial_byte() {
        let data = "0591A96B238A78360ECD43AC62CAC979C4460ED03D780B69DD6FF036B6F79590DC\
                      C8E7E42CA32A54D397F01D19DE250AED0B0D26AA0C3B07DA7D64C2F938065584";
        let data = data.from_hex().unwrap();
        let public_key = PublicKey::new(data.as_slice());
        assert!(public_key.is_none());
    }

    #[test]
    fn test_from_private_key() {
        let data = "6B68589FA737367206B9E97DEE27828B9688FA3D034352DA0E79340B882582F9";
        let data = data.from_hex().unwrap();
        let private_key = PrivateKey::new(data.as_slice()).unwrap();
        let public_key = PublicKey::from_private_key(&private_key);
        let expected = "048E9DD4F17736E54FE6E8C1AA6E784336D0719F4FB726179142497CC7104A969B\
                          A284828FF9AAB80619BDF0AFB70A626B077391768242C300594A25D475068F29";
        let expected = expected.from_hex().unwrap();
        assert_eq!(public_key.get_data(), expected.as_slice());
    }

    #[test]
    fn test_to_address() {
        let data = "0423111FB83A08B04A546F94BC6845E07BCD5105E4738631DCDCE8E8656A9F3405\
                      9FC7368BE3FFB812E0C0BCB4C671CE7EE61B277BC4C1ED0240E6A346E5BBBFC0";
        let data = data.from_hex().unwrap();
        let public_key = PublicKey::new(data.as_slice()).unwrap();
        let address = public_key.to_address();
        let expected = util::base58::decode("1Eii6CZznXKL5qYwEYGdWGYGUFcDm8znL8").unwrap();
        assert_eq!(address.get_data(), expected.as_slice());
    }
}

