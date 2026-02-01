use std::ops::{Shl, BitAnd, BitOr, Not} ;

const ALPHABET: [char; 64] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '+', '/', 
];

const BITS_PER_CHAR: u8 = 6;

fn read_bit<V, P>(val: V, bit_position: P) -> bool
where
    V: Shl<P, Output = V>
        + BitAnd<Output = V>
        + PartialEq
        + From<u8>
{
    (val & (V::from(1u8) << bit_position)) != V::from(0u8)
}

fn write_bit<V, P>(val: V, bit_position: P, on: bool) -> V
where
    V: Shl<P, Output = V>
        + BitAnd<Output = V>
        + BitOr<Output = V>
        + PartialEq
        + From<u8>
        + Not<Output = V>
{
    let res: V;

    if on {
        res = val | (V::from(1u8) << bit_position);
    } else {
        res = val & !(V::from(1u8) << bit_position);
    }

    res
}

pub fn encode(bytes: &[u8]) -> String {
    let mut buffer: u32 = 0;
    let mut buffer_idx: u8 = 0;
    let mut chunks: Vec<u32> = Vec::new();

    for &byte in bytes {
        for i in (0..u8::BITS).rev() {
            let bit_on = read_bit(byte,  i);

            buffer = write_bit(buffer, BITS_PER_CHAR - 1 - buffer_idx, bit_on);

            buffer_idx += 1;

            if buffer_idx == BITS_PER_CHAR {
                chunks.push(buffer);
                buffer = 0;
                buffer_idx = 0;
            }

        }
    }

    if buffer_idx != BITS_PER_CHAR {
        chunks.push(buffer);
    }

    let chars: Vec<char> = chunks.iter().map(|i| ALPHABET[*i as usize]).collect();

    chars.iter().collect::<String>()
}

#[derive(thiserror::Error, Debug)]
enum DecodeError {
    #[error("Invalid Input")]
    InvalidInput
}

pub fn decode(input: &str) -> Result<Vec<u8>, DecodeError> {
    let mut result: Vec<u8> = Vec::new();

    let mut buffer: u8 = 0;
    let mut bits_read: u32 = 0;

    for character in input.chars() {
        let pos = ALPHABET.iter().position(|&e| e == character).ok_or(DecodeError::InvalidInput)?;

        for i in (0..BITS_PER_CHAR).rev() {
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

        assert_eq!(encoded, "YWJjZA");
    }

    #[test]
    fn test_decode() {
        let expected = vec![97, 98, 99, 100]; //"abcd"
        let input = "YWJjZA";

        let decoded = decode(&input).unwrap();

        assert_eq!(decoded, expected);
    }
}
