pub mod packet;

use self::packet::Packet;
pub struct GcmCipher {
    key: [u8; 32],
}

pub struct InvalidData;

impl GcmCipher {
    pub const fn new(key: [u8; 32]) -> GcmCipher {
        GcmCipher { key }
    }

    pub fn encrypt(&self, data: &[u8]) -> Packet {
        todo!();
    }

    pub fn decrypt(&self, packet: Packet) -> Result<Vec<u8>, InvalidData> {
        match self.packet_is_valid(&packet) {
            true => Ok([
                packet.additional_data(),
                self.xor_bit_stream(packet.nonce(), packet.data()).as_slice(),
            ].concat()),
            false => Err(InvalidData),
        }
    }

    /// produces a tag for given data
    pub fn g_hash(
        &self,
        encrypted_data: &[u8],
        additional_data: &[u8],
    ) -> packet::Tag {
        todo!();
    }

    /// verifies that a give packet has not been tampered with
    pub fn packet_is_valid(&self, packet: &Packet) -> bool {
        self.g_hash(packet.data(), packet.additional_data()) == packet.tag()
    }

    /// encrypts/decrypts the data
    fn xor_bit_stream(&self, nonce: packet::Nonce, data: &[u8]) -> Vec<u8> {
        todo!();
    }
}