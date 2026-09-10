//! Serialize `ProgressiveVariableList<FixedVector<u8, M>>` as list of 0x-prefixed hex string.
//!
//! The progressive (EIP-7688) counterpart of [`list_of_hex_fixed_vec`](super::list_of_hex_fixed_vec).
use crate::{FixedVector, ProgressiveVariableList};
use serde::{de::Error, ser::SerializeSeq, Deserializer, Serializer};
use std::marker::PhantomData;
use typenum::Unsigned;

// The element wrappers are identical to the bounded `list_of_hex_fixed_vec`, so reuse them.
pub use super::list_of_hex_fixed_vec::{WrappedListOwned, WrappedListRef};

pub fn serialize<S, M, N>(
    list: &ProgressiveVariableList<FixedVector<u8, M>, N>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    M: Unsigned,
{
    let mut seq = serializer.serialize_seq(Some(list.len()))?;
    for bytes in list {
        seq.serialize_element(&WrappedListRef(bytes))?;
    }
    seq.end()
}

pub struct Visitor<M, N> {
    _phantom_m: PhantomData<M>,
    _phantom_n: PhantomData<N>,
}

impl<M, N> Default for Visitor<M, N> {
    fn default() -> Self {
        Self {
            _phantom_m: PhantomData,
            _phantom_n: PhantomData,
        }
    }
}

impl<'a, M, N> serde::de::Visitor<'a> for Visitor<M, N>
where
    M: Unsigned,
    N: Unsigned,
{
    type Value = ProgressiveVariableList<FixedVector<u8, M>, N>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a list of 0x-prefixed hex bytes")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'a>,
    {
        let mut list = Vec::new();

        while let Some(val) = seq.next_element::<WrappedListOwned<M>>()? {
            list.push(val.0);
        }

        if let Some(max) = Self::Value::max_len() {
            if list.len() > max {
                return Err(A::Error::custom(format!(
                    "ProgressiveVariableList length {} exceeds maximum length {}",
                    list.len(),
                    max
                )));
            }
        }

        Ok(ProgressiveVariableList::new(list))
    }
}

pub fn deserialize<'de, D, M, N>(
    deserializer: D,
) -> Result<ProgressiveVariableList<FixedVector<u8, M>, N>, D::Error>
where
    D: Deserializer<'de>,
    M: Unsigned,
    N: Unsigned,
{
    deserializer.deserialize_seq(Visitor::default())
}
