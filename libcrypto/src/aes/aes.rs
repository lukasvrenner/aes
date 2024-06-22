//! A software implementation of AES, as specified by NIST
//!
//! 128, 192, and 256-bit keys are supported
//!
//! Decryption is not supported because
//! we only ever use AES in counter mode, which only needs encryption
//!
//! Encryption works in 16-byte blocks.
//! This module only provides one-block encryption.
//!
//! Generally, this module will not be used on its own.
//! It is paired with a "mode of operation," such as GCM or CBC.
//!
//! # Examples
//!
//! ```
//! use libcrypto::aes::{AesCipher, Aes128};
//!
//! let mut plain_text = *b"Hello, world!!!!";
//! let key = [
//!     0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15,
//!     0x88, 0x09, 0xcf, 0x4f, 0x3c,
//! ];
//! let cipher = Aes128::new(key);
//! cipher.encrypt_inline(&mut plain_text);
//! let cipher_text = [
//!     0x44, 0x9e, 0xdd, 0xbb, 0xce, 0x73, 0x45, 0x88, 0xd6, 0x7e, 0x1d,
//!     0x8c, 0x54, 0x62, 0x38, 0x14,
//! ];
//! assert_eq!(plain_text, cipher_text);
//! ```

/// The size of a single block
///
/// AES operates on a fixed size
pub const BLOCK_SIZE: usize = 16;

/// a substitution table for the SBox transformation
const S_BOX: [u8; 256] = [
    0x63, 0x7C, 0x77, 0x7B, 0xF2, 0x6B, 0x6F, 0xC5, 0x30, 0x01, 0x67, 0x2B, 0xFE, 0xD7, 0xAB, 0x76,
    0xCA, 0x82, 0xC9, 0x7D, 0xFA, 0x59, 0x47, 0xF0, 0xAD, 0xD4, 0xA2, 0xAF, 0x9C, 0xA4, 0x72, 0xC0,
    0xB7, 0xFD, 0x93, 0x26, 0x36, 0x3F, 0xF7, 0xCC, 0x34, 0xA5, 0xE5, 0xF1, 0x71, 0xD8, 0x31, 0x15,
    0x04, 0xC7, 0x23, 0xC3, 0x18, 0x96, 0x05, 0x9A, 0x07, 0x12, 0x80, 0xE2, 0xEB, 0x27, 0xB2, 0x75,
    0x09, 0x83, 0x2C, 0x1A, 0x1B, 0x6E, 0x5A, 0xA0, 0x52, 0x3B, 0xD6, 0xB3, 0x29, 0xE3, 0x2F, 0x84,
    0x53, 0xD1, 0x00, 0xED, 0x20, 0xFC, 0xB1, 0x5B, 0x6A, 0xCB, 0xBE, 0x39, 0x4A, 0x4C, 0x58, 0xCF,
    0xD0, 0xEF, 0xAA, 0xFB, 0x43, 0x4D, 0x33, 0x85, 0x45, 0xF9, 0x02, 0x7F, 0x50, 0x3C, 0x9F, 0xA8,
    0x51, 0xA3, 0x40, 0x8F, 0x92, 0x9D, 0x38, 0xF5, 0xBC, 0xB6, 0xDA, 0x21, 0x10, 0xFF, 0xF3, 0xD2,
    0xCD, 0x0C, 0x13, 0xEC, 0x5F, 0x97, 0x44, 0x17, 0xC4, 0xA7, 0x7E, 0x3D, 0x64, 0x5D, 0x19, 0x73,
    0x60, 0x81, 0x4F, 0xDC, 0x22, 0x2A, 0x90, 0x88, 0x46, 0xEE, 0xB8, 0x14, 0xDE, 0x5E, 0x0B, 0xDB,
    0xE0, 0x32, 0x3A, 0x0A, 0x49, 0x06, 0x24, 0x5C, 0xC2, 0xD3, 0xAC, 0x62, 0x91, 0x95, 0xE4, 0x79,
    0xE7, 0xC8, 0x37, 0x6D, 0x8D, 0xD5, 0x4E, 0xA9, 0x6C, 0x56, 0xF4, 0xEA, 0x65, 0x7A, 0xAE, 0x08,
    0xBA, 0x78, 0x25, 0x2E, 0x1C, 0xA6, 0xB4, 0xC6, 0xE8, 0xDD, 0x74, 0x1F, 0x4B, 0xBD, 0x8B, 0x8A,
    0x70, 0x3E, 0xB5, 0x66, 0x48, 0x03, 0xF6, 0x0E, 0x61, 0x35, 0x57, 0xB9, 0x86, 0xC1, 0x1D, 0x9E,
    0xE1, 0xF8, 0x98, 0x11, 0x69, 0xD9, 0x8E, 0x94, 0x9B, 0x1E, 0x87, 0xE9, 0xCE, 0x55, 0x28, 0xDF,
    0x8C, 0xA1, 0x89, 0x0D, 0xBF, 0xE6, 0x42, 0x68, 0x41, 0x99, 0x2D, 0x0F, 0xB0, 0x54, 0xBB, 0x16,
];

/// GF(2^8) multiplication table
///
/// If multipling `a * b`, use `a - 1` as `a`
///
/// Most values, including `a = 1`
/// are left out because only a small subset is ever used
const MULT_GF_2_TO_8: [[u8; 256]; 3] = [
    [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
        0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d,
        0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c,
        0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x3b,
        0x3c, 0x3d, 0x3e, 0x3f, 0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a,
        0x4b, 0x4c, 0x4d, 0x4e, 0x4f, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59,
        0x5a, 0x5b, 0x5c, 0x5d, 0x5e, 0x5f, 0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68,
        0x69, 0x6a, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77,
        0x78, 0x79, 0x7a, 0x7b, 0x7c, 0x7d, 0x7e, 0x7f, 0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86,
        0x87, 0x88, 0x89, 0x8a, 0x8b, 0x8c, 0x8d, 0x8e, 0x8f, 0x90, 0x91, 0x92, 0x93, 0x94, 0x95,
        0x96, 0x97, 0x98, 0x99, 0x9a, 0x9b, 0x9c, 0x9d, 0x9e, 0x9f, 0xa0, 0xa1, 0xa2, 0xa3, 0xa4,
        0xa5, 0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xab, 0xac, 0xad, 0xae, 0xaf, 0xb0, 0xb1, 0xb2, 0xb3,
        0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xbb, 0xbc, 0xbd, 0xbe, 0xbf, 0xc0, 0xc1, 0xc2,
        0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xcb, 0xcc, 0xcd, 0xce, 0xcf, 0xd0, 0xd1,
        0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xdb, 0xdc, 0xdd, 0xde, 0xdf, 0xe0,
        0xe1, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea, 0xeb, 0xec, 0xed, 0xee, 0xef,
        0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe,
        0xff,
    ],
    [
        0x00, 0x02, 0x04, 0x06, 0x08, 0x0a, 0x0c, 0x0e, 0x10, 0x12, 0x14, 0x16, 0x18, 0x1a, 0x1c,
        0x1e, 0x20, 0x22, 0x24, 0x26, 0x28, 0x2a, 0x2c, 0x2e, 0x30, 0x32, 0x34, 0x36, 0x38, 0x3a,
        0x3c, 0x3e, 0x40, 0x42, 0x44, 0x46, 0x48, 0x4a, 0x4c, 0x4e, 0x50, 0x52, 0x54, 0x56, 0x58,
        0x5a, 0x5c, 0x5e, 0x60, 0x62, 0x64, 0x66, 0x68, 0x6a, 0x6c, 0x6e, 0x70, 0x72, 0x74, 0x76,
        0x78, 0x7a, 0x7c, 0x7e, 0x80, 0x82, 0x84, 0x86, 0x88, 0x8a, 0x8c, 0x8e, 0x90, 0x92, 0x94,
        0x96, 0x98, 0x9a, 0x9c, 0x9e, 0xa0, 0xa2, 0xa4, 0xa6, 0xa8, 0xaa, 0xac, 0xae, 0xb0, 0xb2,
        0xb4, 0xb6, 0xb8, 0xba, 0xbc, 0xbe, 0xc0, 0xc2, 0xc4, 0xc6, 0xc8, 0xca, 0xcc, 0xce, 0xd0,
        0xd2, 0xd4, 0xd6, 0xd8, 0xda, 0xdc, 0xde, 0xe0, 0xe2, 0xe4, 0xe6, 0xe8, 0xea, 0xec, 0xee,
        0xf0, 0xf2, 0xf4, 0xf6, 0xf8, 0xfa, 0xfc, 0xfe, 0x1b, 0x19, 0x1f, 0x1d, 0x13, 0x11, 0x17,
        0x15, 0x0b, 0x09, 0x0f, 0x0d, 0x03, 0x01, 0x07, 0x05, 0x3b, 0x39, 0x3f, 0x3d, 0x33, 0x31,
        0x37, 0x35, 0x2b, 0x29, 0x2f, 0x2d, 0x23, 0x21, 0x27, 0x25, 0x5b, 0x59, 0x5f, 0x5d, 0x53,
        0x51, 0x57, 0x55, 0x4b, 0x49, 0x4f, 0x4d, 0x43, 0x41, 0x47, 0x45, 0x7b, 0x79, 0x7f, 0x7d,
        0x73, 0x71, 0x77, 0x75, 0x6b, 0x69, 0x6f, 0x6d, 0x63, 0x61, 0x67, 0x65, 0x9b, 0x99, 0x9f,
        0x9d, 0x93, 0x91, 0x97, 0x95, 0x8b, 0x89, 0x8f, 0x8d, 0x83, 0x81, 0x87, 0x85, 0xbb, 0xb9,
        0xbf, 0xbd, 0xb3, 0xb1, 0xb7, 0xb5, 0xab, 0xa9, 0xaf, 0xad, 0xa3, 0xa1, 0xa7, 0xa5, 0xdb,
        0xd9, 0xdf, 0xdd, 0xd3, 0xd1, 0xd7, 0xd5, 0xcb, 0xc9, 0xcf, 0xcd, 0xc3, 0xc1, 0xc7, 0xc5,
        0xfb, 0xf9, 0xff, 0xfd, 0xf3, 0xf1, 0xf7, 0xf5, 0xeb, 0xe9, 0xef, 0xed, 0xe3, 0xe1, 0xe7,
        0xe5,
    ],
    [
        0x00, 0x03, 0x06, 0x05, 0x0c, 0x0f, 0x0a, 0x09, 0x18, 0x1b, 0x1e, 0x1d, 0x14, 0x17, 0x12,
        0x11, 0x30, 0x33, 0x36, 0x35, 0x3c, 0x3f, 0x3a, 0x39, 0x28, 0x2b, 0x2e, 0x2d, 0x24, 0x27,
        0x22, 0x21, 0x60, 0x63, 0x66, 0x65, 0x6c, 0x6f, 0x6a, 0x69, 0x78, 0x7b, 0x7e, 0x7d, 0x74,
        0x77, 0x72, 0x71, 0x50, 0x53, 0x56, 0x55, 0x5c, 0x5f, 0x5a, 0x59, 0x48, 0x4b, 0x4e, 0x4d,
        0x44, 0x47, 0x42, 0x41, 0xc0, 0xc3, 0xc6, 0xc5, 0xcc, 0xcf, 0xca, 0xc9, 0xd8, 0xdb, 0xde,
        0xdd, 0xd4, 0xd7, 0xd2, 0xd1, 0xf0, 0xf3, 0xf6, 0xf5, 0xfc, 0xff, 0xfa, 0xf9, 0xe8, 0xeb,
        0xee, 0xed, 0xe4, 0xe7, 0xe2, 0xe1, 0xa0, 0xa3, 0xa6, 0xa5, 0xac, 0xaf, 0xaa, 0xa9, 0xb8,
        0xbb, 0xbe, 0xbd, 0xb4, 0xb7, 0xb2, 0xb1, 0x90, 0x93, 0x96, 0x95, 0x9c, 0x9f, 0x9a, 0x99,
        0x88, 0x8b, 0x8e, 0x8d, 0x84, 0x87, 0x82, 0x81, 0x9b, 0x98, 0x9d, 0x9e, 0x97, 0x94, 0x91,
        0x92, 0x83, 0x80, 0x85, 0x86, 0x8f, 0x8c, 0x89, 0x8a, 0xab, 0xa8, 0xad, 0xae, 0xa7, 0xa4,
        0xa1, 0xa2, 0xb3, 0xb0, 0xb5, 0xb6, 0xbf, 0xbc, 0xb9, 0xba, 0xfb, 0xf8, 0xfd, 0xfe, 0xf7,
        0xf4, 0xf1, 0xf2, 0xe3, 0xe0, 0xe5, 0xe6, 0xef, 0xec, 0xe9, 0xea, 0xcb, 0xc8, 0xcd, 0xce,
        0xc7, 0xc4, 0xc1, 0xc2, 0xd3, 0xd0, 0xd5, 0xd6, 0xdf, 0xdc, 0xd9, 0xda, 0x5b, 0x58, 0x5d,
        0x5e, 0x57, 0x54, 0x51, 0x52, 0x43, 0x40, 0x45, 0x46, 0x4f, 0x4c, 0x49, 0x4a, 0x6b, 0x68,
        0x6d, 0x6e, 0x67, 0x64, 0x61, 0x62, 0x73, 0x70, 0x75, 0x76, 0x7f, 0x7c, 0x79, 0x7a, 0x3b,
        0x38, 0x3d, 0x3e, 0x37, 0x34, 0x31, 0x32, 0x23, 0x20, 0x25, 0x26, 0x2f, 0x2c, 0x29, 0x2a,
        0x0b, 0x08, 0x0d, 0x0e, 0x07, 0x04, 0x01, 0x02, 0x13, 0x10, 0x15, 0x16, 0x1f, 0x1c, 0x19,
        0x1a,
    ],
];

/// a lookup table used for key key expansion
const R_CON: [u32; 256] = [
    0x8d, 0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36, 0x6c, 0xd8, 0xab, 0x4d, 0x9a,
    0x2f, 0x5e, 0xbc, 0x63, 0xc6, 0x97, 0x35, 0x6a, 0xd4, 0xb3, 0x7d, 0xfa, 0xef, 0xc5, 0x91, 0x39,
    0x72, 0xe4, 0xd3, 0xbd, 0x61, 0xc2, 0x9f, 0x25, 0x4a, 0x94, 0x33, 0x66, 0xcc, 0x83, 0x1d, 0x3a,
    0x74, 0xe8, 0xcb, 0x8d, 0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36, 0x6c, 0xd8,
    0xab, 0x4d, 0x9a, 0x2f, 0x5e, 0xbc, 0x63, 0xc6, 0x97, 0x35, 0x6a, 0xd4, 0xb3, 0x7d, 0xfa, 0xef,
    0xc5, 0x91, 0x39, 0x72, 0xe4, 0xd3, 0xbd, 0x61, 0xc2, 0x9f, 0x25, 0x4a, 0x94, 0x33, 0x66, 0xcc,
    0x83, 0x1d, 0x3a, 0x74, 0xe8, 0xcb, 0x8d, 0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b,
    0x36, 0x6c, 0xd8, 0xab, 0x4d, 0x9a, 0x2f, 0x5e, 0xbc, 0x63, 0xc6, 0x97, 0x35, 0x6a, 0xd4, 0xb3,
    0x7d, 0xfa, 0xef, 0xc5, 0x91, 0x39, 0x72, 0xe4, 0xd3, 0xbd, 0x61, 0xc2, 0x9f, 0x25, 0x4a, 0x94,
    0x33, 0x66, 0xcc, 0x83, 0x1d, 0x3a, 0x74, 0xe8, 0xcb, 0x8d, 0x01, 0x02, 0x04, 0x08, 0x10, 0x20,
    0x40, 0x80, 0x1b, 0x36, 0x6c, 0xd8, 0xab, 0x4d, 0x9a, 0x2f, 0x5e, 0xbc, 0x63, 0xc6, 0x97, 0x35,
    0x6a, 0xd4, 0xb3, 0x7d, 0xfa, 0xef, 0xc5, 0x91, 0x39, 0x72, 0xe4, 0xd3, 0xbd, 0x61, 0xc2, 0x9f,
    0x25, 0x4a, 0x94, 0x33, 0x66, 0xcc, 0x83, 0x1d, 0x3a, 0x74, 0xe8, 0xcb, 0x8d, 0x01, 0x02, 0x04,
    0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36, 0x6c, 0xd8, 0xab, 0x4d, 0x9a, 0x2f, 0x5e, 0xbc, 0x63,
    0xc6, 0x97, 0x35, 0x6a, 0xd4, 0xb3, 0x7d, 0xfa, 0xef, 0xc5, 0x91, 0x39, 0x72, 0xe4, 0xd3, 0xbd,
    0x61, 0xc2, 0x9f, 0x25, 0x4a, 0x94, 0x33, 0x66, 0xcc, 0x83, 0x1d, 0x3a, 0x74, 0xe8, 0xcb, 0x8d,
];

/// AES-128 encryption
pub struct Aes128 {
    round_keys: [[u8; BLOCK_SIZE]; Self::NUM_ROUNDS + 1],
}

/// AES-192 encryption
pub struct Aes192 {
    round_keys: [[u8; BLOCK_SIZE]; Self::NUM_ROUNDS + 1],
}

/// AES-256 encryption
pub struct Aes256 {
    round_keys: [[u8; BLOCK_SIZE]; Self::NUM_ROUNDS + 1],
}

/// A common interface for AES ciphers
pub trait AesCipher {
    /// The length of the key, in bytes
    ///
    /// For AES-128, this is 16.
    /// For AES-192, this is 24.
    /// For AES-256, this is 32.
    const KEY_SIZE: usize;

    /// The number of rounds AES will loop through
    ///
    /// For AES-128, this is 10. \
    /// For AES-192, this is 12. \
    /// For AES-256, this is 16.
    const NUM_ROUNDS: usize;

    /// The number of 32-bit "words" in the key
    const NUM_KEY_WORDS: usize = Self::KEY_SIZE / 4;
    /// The type used to represent a key.
    ///
    /// Note: this will be auto-implemented
    /// once trait type defaults are stabilized.
    type Key;

    /// Encrypts `block` inline, mutating `block`
    fn encrypt_inline(&self, block: &mut [u8; BLOCK_SIZE]);

    /// Copies `block` into a new buffer and encrypts the buffer
    fn encrypt(&self, block: &[u8; BLOCK_SIZE]) -> [u8; BLOCK_SIZE] {
        let mut buffer = *block;
        self.encrypt_inline(&mut buffer);
        buffer
    }

    /// Create a new cipher using `key`
    fn new(key: Self::Key) -> Self;
}

// we can't use a trait because the input and
// return types are dependant on the specific implementor
macro_rules! impl_expand_key {
    ($cipher:ty) => {
        impl $cipher {
            fn expand_key(key: [u8; Self::KEY_SIZE]) -> [[u8; BLOCK_SIZE]; Self::NUM_ROUNDS + 1] {
                // endianness doesn't matter so long as byte order is maintained
                // SAFETY: integer arrays can be safely cast
                let key: [u32; Self::NUM_KEY_WORDS] = unsafe { std::mem::transmute(key) };
                let mut expanded_keys = [0u32; 4 * (Self::NUM_ROUNDS + 1)];

                expanded_keys[0..key.len()].copy_from_slice(&key);
                for i in Self::NUM_KEY_WORDS..expanded_keys.len() {
                    let mut temp = expanded_keys[i - 1];
                    temp = match i % Self::NUM_KEY_WORDS {
                        0 => sub_word(rotate_word(temp)) ^ R_CON[i / Self::NUM_KEY_WORDS],
                        4 if const { Self::NUM_KEY_WORDS > 6 } => sub_word(temp),
                        _ => temp,
                    };
                    expanded_keys[i] = expanded_keys[i - Self::NUM_KEY_WORDS] ^ temp;
                }
                // endianness doesn't matter so long as byte order is maintained
                // SAFETY: integer arrays can be safely cast
                unsafe { std::mem::transmute(expanded_keys) }
            }
        }
    };
}

impl_expand_key!(Aes128);
impl_expand_key!(Aes192);
impl_expand_key!(Aes256);

// we can add this as an auto-implemented trait function for AesCipher
// once trait type defaults is stabilized
macro_rules! impl_aes_cipher {
    ($cipher:ty, $key_size:literal, $num_rounds:literal) => {
        impl AesCipher for $cipher {
            const KEY_SIZE: usize = $key_size;
            const NUM_ROUNDS: usize = $num_rounds;

            type Key = [u8; Self::KEY_SIZE];

            fn encrypt_inline(&self, block: &mut [u8; BLOCK_SIZE]) {
                add_round_key(block, self.round_keys[0]);
                for round_key in self.round_keys[1..self.round_keys.len() - 1].iter() {
                    sub_bytes(block);
                    shift_rows(block);
                    mix_columns(block);
                    add_round_key(block, *round_key);
                }
                sub_bytes(block);
                shift_rows(block);
                add_round_key(block, self.round_keys[Self::NUM_ROUNDS]);
            }

            fn new(key: Self::Key) -> Self {
                Self {
                    round_keys: Self::expand_key(key),
                }
            }
        }
    };
}

impl_aes_cipher!(Aes128, 16, 10);
impl_aes_cipher!(Aes192, 24, 12);
impl_aes_cipher!(Aes256, 32, 14);

#[inline]
fn add_round_key(state: &mut [u8; BLOCK_SIZE], round_key: [u8; BLOCK_SIZE]) {
    for (state_byte, key_byte) in state.iter_mut().zip(round_key) {
        *state_byte ^= key_byte;
    }
}

#[inline]
fn sub_bytes(state: &mut [u8; BLOCK_SIZE]) {
    for byte in state {
        *byte = s_box(*byte);
    }
}

#[inline]
fn shift_rows(state: &mut [u8; BLOCK_SIZE]) {
    // TODO: use uninitialized memory if necessary
    let mut auxiliary = [0u8; 4];
    for row in 0..4 {
        for col in 0..4 {
            auxiliary[col] = state[col * 4 + row];
        }
        for col in 0..4 {
            state[col * 4 + row] = auxiliary[(row + col) % 4];
        }
    }
}

#[inline]
fn mix_columns(state: &mut [u8; BLOCK_SIZE]) {
    let mult_matrix = [
        [0x01, 0x02, 0x00, 0x00],
        [0x00, 0x01, 0x02, 0x00],
        [0x00, 0x00, 0x01, 0x02],
        [0x02, 0x00, 0x00, 0x01],
    ];
    // TODO: use `array_chunks` once stabilized
    for col in state.chunks_exact_mut(4) {
        let auxiliary: [u8; 4] = col.try_into().unwrap();

        for row in 0..4 {
            let mut byte = 0u8;
            for i in 0..4 {
                byte ^= MULT_GF_2_TO_8[mult_matrix[row][i]][auxiliary[i] as usize];
            }
            col[row] = byte;
        }
    }
}

#[inline]
const fn s_box(byte: u8) -> u8 {
    S_BOX[byte as usize]
}

#[inline]
fn sub_word(word: u32) -> u32 {
    // endianness doesn't matter so long as byte order is maintained
    u32::from_ne_bytes(word.to_ne_bytes().map(s_box))
}

#[inline]
const fn rotate_word(word: u32) -> u32 {
    word.rotate_right(8)
}

#[cfg(test)]
mod tests {
    use super::{Aes128, Aes256, AesCipher, BLOCK_SIZE};

    #[test]
    fn add_round_key() {
        let mut state: [u8; 16] = [
            0x32, 0x43, 0xf6, 0xa8, 0x88, 0x5a, 0x30, 0x8d, 0x31, 0x31, 0x98, 0xa2, 0xe0, 0x37,
            0x07, 0x34,
        ];
        let round_key: [u8; 16] = [
            0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf,
            0x4f, 0x3c,
        ];
        let output_state: [u8; 16] = [
            0x19, 0x3d, 0xe3, 0xbe, 0xa0, 0xf4, 0xe2, 0x2b, 0x9a, 0xc6, 0x8d, 0x2a, 0xe9, 0xf8,
            0x48, 0x08,
        ];
        super::add_round_key(&mut state, round_key);
        assert_eq!(state, output_state);
    }

    #[test]
    fn sub_bytes() {
        let mut state: [u8; 16] = [
            0x19, 0x3d, 0xe3, 0xbe, 0xa0, 0xf4, 0xe2, 0x2b, 0x9a, 0xc6, 0x8d, 0x2a, 0xe9, 0xf8,
            0x48, 0x08,
        ];
        let output_state: [u8; 16] = [
            0xd4, 0x27, 0x11, 0xae, 0xe0, 0xbf, 0x98, 0xf1, 0xb8, 0xb4, 0x5d, 0xe5, 0x1e, 0x41,
            0x52, 0x30,
        ];
        super::sub_bytes(&mut state);
        assert_eq!(state, output_state);
    }

    #[test]
    fn shift_rows() {
        let mut state: [u8; 16] = [
            0xd4, 0x27, 0x11, 0xae, 0xe0, 0xbf, 0x98, 0xf1, 0xb8, 0xb4, 0x5d, 0xe5, 0x1e, 0x41,
            0x52, 0x30,
        ];
        let output_state: [u8; 16] = [
            0xd4, 0xbf, 0x5d, 0x30, 0xe0, 0xb4, 0x52, 0xae, 0xb8, 0x41, 0x11, 0xf1, 0x1e, 0x27,
            0x98, 0xe5,
        ];
        super::shift_rows(&mut state);
        assert_eq!(state, output_state);
    }

    #[test]
    fn mix_columns() {
        let mut state: [u8; 16] = [
            0xd4, 0xbf, 0x5d, 0x30, 0xe0, 0xb4, 0x52, 0xae, 0xb8, 0x41, 0x11, 0xf1, 0x1e, 0x27,
            0x98, 0xe5,
        ];
        let output_state: [u8; 16] = [
            0x04, 0x66, 0x81, 0xe5, 0xe0, 0xcb, 0x19, 0x9a, 0x48, 0xf8, 0xd3, 0x7a, 0x28, 0x06,
            0x26, 0x4c,
        ];
        super::mix_columns(&mut state);
        assert_eq!(state, output_state);
    }

    #[test]
    fn key_expansion_128() {
        let key: [u8; Aes128::KEY_SIZE] = [
            0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf,
            0x4f, 0x3c,
        ];
        let expanded_keys: [[u8; BLOCK_SIZE]; Aes128::NUM_ROUNDS + 1] = [
            [
                0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf,
                0x4f, 0x3c,
            ],
            [
                0xa0, 0xfa, 0xfe, 0x17, 0x88, 0x54, 0x2c, 0xb1, 0x23, 0xa3, 0x39, 0x39, 0x2a, 0x6c,
                0x76, 0x05,
            ],
            [
                0xf2, 0xc2, 0x95, 0xf2, 0x7a, 0x96, 0xb9, 0x43, 0x59, 0x35, 0x80, 0x7a, 0x73, 0x59,
                0xf6, 0x7f,
            ],
            [
                0x3d, 0x80, 0x47, 0x7d, 0x47, 0x16, 0xfe, 0x3e, 0x1e, 0x23, 0x7e, 0x44, 0x6d, 0x7a,
                0x88, 0x3b,
            ],
            [
                0xef, 0x44, 0xa5, 0x41, 0xa8, 0x52, 0x5b, 0x7f, 0xb6, 0x71, 0x25, 0x3b, 0xdb, 0x0b,
                0xad, 0x00,
            ],
            [
                0xd4, 0xd1, 0xc6, 0xf8, 0x7c, 0x83, 0x9d, 0x87, 0xca, 0xf2, 0xb8, 0xbc, 0x11, 0xf9,
                0x15, 0xbc,
            ],
            [
                0x6d, 0x88, 0xa3, 0x7a, 0x11, 0x0b, 0x3e, 0xfd, 0xdb, 0xf9, 0x86, 0x41, 0xca, 0x00,
                0x93, 0xfd,
            ],
            [
                0x4e, 0x54, 0xf7, 0x0e, 0x5f, 0x5f, 0xc9, 0xf3, 0x84, 0xa6, 0x4f, 0xb2, 0x4e, 0xa6,
                0xdc, 0x4f,
            ],
            [
                0xea, 0xd2, 0x73, 0x21, 0xb5, 0x8d, 0xba, 0xd2, 0x31, 0x2b, 0xf5, 0x60, 0x7f, 0x8d,
                0x29, 0x2f,
            ],
            [
                0xac, 0x77, 0x66, 0xf3, 0x19, 0xfa, 0xdc, 0x21, 0x28, 0xd1, 0x29, 0x41, 0x57, 0x5c,
                0x00, 0x6e,
            ],
            [
                0xd0, 0x14, 0xf9, 0xa8, 0xc9, 0xee, 0x25, 0x89, 0xe1, 0x3f, 0x0c, 0xc8, 0xb6, 0x63,
                0x0c, 0xa6,
            ],
        ];
        let cipher = Aes128::new(key);
        assert_eq!(cipher.round_keys, expanded_keys);
    }

    #[test]
    fn key_expansion_256() {
        let key: [u8; Aes256::KEY_SIZE] = [
            0x60, 0x3d, 0xeb, 0x10, 0x15, 0xca, 0x71, 0xbe, 0x2b, 0x73, 0xae, 0xf0, 0x85, 0x7d,
            0x77, 0x81, 0x1f, 0x35, 0x2c, 0x07, 0x3b, 0x61, 0x08, 0xd7, 0x2d, 0x98, 0x10, 0xa3,
            0x09, 0x14, 0xdf, 0xf4,
        ];
        // fn fails() -> [[u8; 16]; 15] {
        // todo!();
        // }
        let expanded_keys: [[u8; BLOCK_SIZE]; Aes256::NUM_ROUNDS + 1] = [
            [
                0x60, 0x3d, 0xeb, 0x10, 0x15, 0xca, 0x71, 0xbe, 0x2b, 0x73, 0xae, 0xf0, 0x85, 0x7d,
                0x77, 0x81,
            ],
            [
                0x1f, 0x35, 0x2c, 0x07, 0x3b, 0x61, 0x08, 0xd7, 0x2d, 0x98, 0x10, 0xa3, 0x09, 0x14,
                0xdf, 0xf4,
            ],
            [
                0x9b, 0xa3, 0x54, 0x11, 0x8e, 0x69, 0x25, 0xaf, 0xa5, 0x1a, 0x8b, 0x5f, 0x20, 0x67,
                0xfc, 0xde,
            ],
            [
                0xa8, 0xb0, 0x9c, 0x1a, 0x93, 0xd1, 0x94, 0xcd, 0xbe, 0x49, 0x84, 0x6e, 0xb7, 0x5d,
                0x5b, 0x9a,
            ],
            [
                0xd5, 0x9a, 0xec, 0xb8, 0x5b, 0xf3, 0xc9, 0x17, 0xfe, 0xe9, 0x42, 0x48, 0xde, 0x8e,
                0xbe, 0x96,
            ],
            [
                0xb5, 0xa9, 0x32, 0x8a, 0x26, 0x78, 0xa6, 0x47, 0x98, 0x31, 0x22, 0x29, 0x2f, 0x6c,
                0x79, 0xb3,
            ],
            [
                0x81, 0x2c, 0x81, 0xad, 0xda, 0xdf, 0x48, 0xba, 0x24, 0x36, 0x0a, 0xf2, 0xfa, 0xb8,
                0xb4, 0x64,
            ],
            [
                0x98, 0xc5, 0xbf, 0xc9, 0xbe, 0xbd, 0x19, 0x8e, 0x26, 0x8c, 0x3b, 0xa7, 0x09, 0xe0,
                0x42, 0x14,
            ],
            [
                0x68, 0x00, 0x7b, 0xac, 0xb2, 0xdf, 0x33, 0x16, 0x96, 0xe9, 0x39, 0xe4, 0x6c, 0x51,
                0x8d, 0x80,
            ],
            [
                0xc8, 0x14, 0xe2, 0x04, 0x76, 0xa9, 0xfb, 0x8a, 0x50, 0x25, 0xc0, 0x2d, 0x59, 0xc5,
                0x82, 0x39,
            ],
            [
                0xde, 0x13, 0x69, 0x67, 0x6c, 0xcc, 0x5a, 0x71, 0xfa, 0x25, 0x63, 0x95, 0x96, 0x74,
                0xee, 0x15,
            ],
            [
                0x58, 0x86, 0xca, 0x5d, 0x2e, 0x2f, 0x31, 0xd7, 0x7e, 0x0a, 0xf1, 0xfa, 0x27, 0xcf,
                0x73, 0xc3,
            ],
            [
                0x74, 0x9c, 0x47, 0xab, 0x18, 0x50, 0x1d, 0xda, 0xe2, 0x75, 0x7e, 0x4f, 0x74, 0x01,
                0x90, 0x5a,
            ],
            [
                0xca, 0xfa, 0xaa, 0xe3, 0xe4, 0xd5, 0x9b, 0x34, 0x9a, 0xdf, 0x6a, 0xce, 0xbd, 0x10,
                0x19, 0x0d,
            ],
            [
                0xfe, 0x48, 0x90, 0xd1, 0xe6, 0x18, 0x8d, 0x0b, 0x04, 0x6d, 0xf3, 0x44, 0x70, 0x6c,
                0x63, 0x1e,
            ],
        ];
        assert_eq!(super::Aes256::expand_key(key), expanded_keys);
    }

    #[test]
    fn encrypt_128() {
        let mut plain_text = [
            0x32, 0x43, 0xf6, 0xa8, 0x88, 0x5a, 0x30, 0x8d, 0x31, 0x31, 0x98, 0xa2, 0xe0, 0x37,
            0x07, 0x34,
        ];
        let key = [
            0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf,
            0x4f, 0x3c,
        ];
        let cipher_text = [
            0x39, 0x25, 0x84, 0x1d, 0x02, 0xdc, 0x09, 0xfb, 0xdc, 0x11, 0x85, 0x97, 0x19, 0x6a,
            0x0b, 0x32,
        ];
        let cipher = Aes128::new(key);
        cipher.encrypt_inline(&mut plain_text);
        assert_eq!(plain_text, cipher_text);
    }

    #[test]
    fn encrypt_256() {
        let key = [0u8; 32];
        let mut plain_text: [u8; 16] = [0xff; 16];
        let cipher_text: [u8; 16] = [
            0xac, 0xda, 0xce, 0x80, 0x78, 0xa3, 0x2b, 0x1a, 0x18, 0x2b, 0xfa, 0x49, 0x87, 0xca,
            0x13, 0x47,
        ];
        let cipher = Aes256::new(key);
        cipher.encrypt_inline(&mut plain_text);
        assert_eq!(plain_text, cipher_text);
    }
}
