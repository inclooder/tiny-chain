use std::ops::{Shl, BitAnd, BitOr, Not};

pub fn read_bit<V, P>(val: V, bit_position: P) -> bool
where
    V: Shl<P, Output = V>
        + BitAnd<Output = V>
        + PartialEq
        + From<u8>
{
    (val & (V::from(1u8) << bit_position)) != V::from(0u8)
}

pub fn write_bit<V, P>(val: V, bit_position: P, on: bool) -> V
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

