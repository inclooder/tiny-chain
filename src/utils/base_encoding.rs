use crate::utils::bit_manipulation::{read_bit, write_bit};

pub const ALPHABET: [char; 57] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'm', 'n', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    '1', '2', '3', '4', '5', '6', '7', '8', '9', 
];

pub fn encode(bytes: &[u8]) -> String {
    encode_with_alphabet(bytes, &ALPHABET)
}

pub fn decode(input: &str) -> Result<Vec<u8>, DecodeError> {
    decode_with_alphabet(input, &ALPHABET)
}

pub fn encode_with_alphabet(bytes: &[u8], alphabet: &[char]) -> String {
    let mut buffer: u32 = 0;
    let mut buffer_idx: u8 = 0;
    let mut chunks: Vec<u32> = Vec::new();

    let bits_per_char = (alphabet.len() as f32).log2() as u8;

    for &byte in bytes {
        for i in (0..u8::BITS).rev() {
            let bit_on = read_bit(byte,  i);

            buffer = write_bit(buffer, bits_per_char - 1 - buffer_idx, bit_on);

            buffer_idx += 1;

            if buffer_idx == bits_per_char {
                chunks.push(buffer);
                buffer = 0;
                buffer_idx = 0;
            }

        }
    }

    if buffer_idx != bits_per_char {
        chunks.push(buffer);
    }

    let chars: Vec<char> = chunks.iter().map(|i| alphabet[*i as usize]).collect();

    chars.iter().collect::<String>()
}

#[derive(thiserror::Error, Debug)]
pub enum DecodeError {
    #[error("Invalid Input")]
    InvalidInput
}

pub fn decode_with_alphabet(input: &str, alphabet: &[char]) -> Result<Vec<u8>, DecodeError> {
    let mut result: Vec<u8> = Vec::new();

    let mut buffer: u8 = 0;
    let mut bits_read: u32 = 0;

    let bits_per_char = (alphabet.len() as f32).log2() as u8;

    for character in input.chars() {
        let pos = alphabet.iter().position(|&e| e == character).ok_or(DecodeError::InvalidInput)?;

        for i in (0..bits_per_char).rev() {
            let bit_on = read_bit(pos, i);

            buffer = write_bit(buffer, u8::BITS - 1 - bits_read, bit_on);

            bits_read += 1;

            if bits_read == u8::BITS {
                result.push(buffer);
                buffer = 0;
                bits_read = 0;
            }
        }
    }

    Ok(result)
}


#[cfg(test)]

mod test {
    use super::*;

    #[test]
    fn test_encode() {
        let input = "abcd";
        let bytes: Vec<u8> = input.bytes().collect();
        let encoded = encode(&bytes);

        assert_eq!(encoded, "NFTGGbA");
    }

    #[test]
    fn test_decode() {
        let input = "NFTGGbA";
        let decoded = decode(&input).unwrap();
        let expected: Vec<u8> = "abcd".bytes().collect();

        assert_eq!(decoded, expected);
    }
}
