//! A software implementation of SHA256

const BLOCK_SIZE: usize = 64;
const HASH_SIZE: usize = 32;

/// The first 32 bits of the fractional parts of
/// the cube roots of the first 64 prime numbers
// TODO: name this something useful
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1,
    0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
    0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786,
    0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147,
    0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
    0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a,
    0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
    0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

// TODO: name this something useful
fn ch(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (!x & z)
}

// TODO: name this something useful
fn maj(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
}

// TODO: name this something useful
fn sigma_0(x: u32) -> u32 {
    x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
}

// TODO: name this something useful
fn sigma_1(x: u32) -> u32 {
    x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
}

// TODO: name this something useful
fn little_sigma_0(x: u32) -> u32 {
    x.rotate_right(7) ^ x.rotate_right(18) ^ x >> 3
}

// TODO: name this something useful
fn little_sigma_1(x: u32) -> u32 {
    x.rotate_right(17) ^ x.rotate_right(19) ^ x >> 10
}

/// Add padding to `msg`, to ensure `msg.len()` is a multiple of 512
fn pad_message(msg: &[u8]) -> Vec<u8> {
    let padding_size =
        (BLOCK_SIZE - BLOCK_SIZE % (msg.len() + 10)) % BLOCK_SIZE;
    let mut padded_msg = msg.to_vec();
    padded_msg.push(0x80);
    padded_msg.resize(padded_msg.len() + padding_size, 0);
    padded_msg.extend_from_slice(&(msg.len() as u64 * 8).to_be_bytes());
    debug_assert_eq!(padded_msg.len() % BLOCK_SIZE, 0);
    padded_msg
}

/// Calculates a 256-bit hash of `msg`
///
/// # Panics
///
/// This function will panic if `msg.len()` is not a multiple of 64
pub fn hash(msg: &[u8]) -> [u8; HASH_SIZE] {
    let padded_msg = pad_message(msg);
    let mut hash: [u32; HASH_SIZE / 4] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c,
        0x1f83d9ab, 0x5be0cd19,
    ];
    padded_msg
        .chunks_exact(BLOCK_SIZE)
        .map(|block| be_bytes_to_u32_array(block.try_into().unwrap()))
        .for_each(|block| update_hash(&mut hash, &block));
    to_be_bytes_from_hash(hash)
}

/// Updates `hash` with the next block (`next_block`)
fn update_hash(
    hash: &mut [u32; HASH_SIZE / 4],
    next_block: &[u32; BLOCK_SIZE / 4],
) {
    let mut message_schedule = [0u32; 64];
    message_schedule[..next_block.len()].copy_from_slice(next_block);

    for i in 16..message_schedule.len() {
        message_schedule[i] = little_sigma_1(message_schedule[i - 2])
            .wrapping_add(message_schedule[i - 7])
            .wrapping_add(little_sigma_0(message_schedule[i - 15]))
            .wrapping_add(message_schedule[i - 16]);
    }

    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = *hash;

    for i in 0..64 {
        let temp1 = h
            .wrapping_add(sigma_1(e))
            .wrapping_add(ch(e, f, g))
            .wrapping_add(K[i])
            .wrapping_add(message_schedule[i]);
        let temp2 = sigma_0(a).wrapping_add(maj(a, b, c));
        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(temp1);
        d = c;
        c = b;
        b = a;
        a = temp1.wrapping_add(temp2);
        println!("{:#x}", d);
    }
    hash[0] = hash[0].wrapping_add(a);
    hash[1] = hash[1].wrapping_add(b);
    hash[2] = hash[2].wrapping_add(c);
    hash[3] = hash[3].wrapping_add(d);
    hash[4] = hash[4].wrapping_add(e);
    hash[5] = hash[5].wrapping_add(f);
    hash[6] = hash[6].wrapping_add(g);
    hash[7] = hash[7].wrapping_add(h);
}

fn be_bytes_to_u32_array(bytes: &[u8; BLOCK_SIZE]) -> [u32; BLOCK_SIZE / 4] {
    let mut as_u32 = [0u32; BLOCK_SIZE / 4];
    for (index, int) in as_u32.iter_mut().enumerate() {
        *int = u32::from_be_bytes(bytes[index * 4..][..4].try_into().unwrap());
    }
    as_u32
}

fn to_be_bytes_from_hash(array: [u32; HASH_SIZE / 4]) -> [u8; HASH_SIZE] {
    let mut as_bytes = [0u8; HASH_SIZE];
    for (index, int) in array.iter().enumerate() {
        as_bytes[index * 4..][..4].copy_from_slice(&int.to_be_bytes());
    }
    as_bytes
}

#[cfg(test)]
mod tests {

    #[test]
    fn padding() {
        let msg = b"abc";
        let block: [u8; super::BLOCK_SIZE] = [
            0x61, 0x62, 0x63, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18,
        ];
        assert_eq!(super::pad_message(msg), block);
    }

    #[test]
    fn sha256() {
        let msg = b"abc";
        let digest: [u8; super::HASH_SIZE] = [
            0xBA, 0x78, 0x16, 0xBF, 0x8F, 0x01, 0xCF, 0xEA, 0x41, 0x41, 0x40,
            0xDE, 0x5D, 0xAE, 0x22, 0x23, 0xB0, 0x03, 0x61, 0xA3, 0x96, 0x17,
            0x7A, 0x9C, 0xB4, 0x10, 0xFF, 0x61, 0xF2, 0x00, 0x15, 0xAD,
        ];
        assert_eq!(super::hash(msg), digest);
    }
}