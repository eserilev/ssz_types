use crate::{typenum::Unsigned, FixedVector, ProgressiveVariableList};
use context_deserialize::ContextDeserialize;
use serde::de::{Deserializer, Error};

impl<'de, C, T, N> ContextDeserialize<'de, C> for FixedVector<T, N>
where
    T: ContextDeserialize<'de, C>,
    N: Unsigned,
    C: Clone,
{
    fn context_deserialize<D>(deserializer: D, context: C) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec = Vec::<T>::context_deserialize(deserializer, context)?;
        FixedVector::new(vec).map_err(|e| D::Error::custom(format!("{:?}", e)))
    }
}

impl<'de, C, T, N> ContextDeserialize<'de, C> for ProgressiveVariableList<T, N>
where
    T: ContextDeserialize<'de, C>,
    N: Unsigned,
    C: Clone,
{
    fn context_deserialize<D>(deserializer: D, context: C) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec = Vec::<T>::context_deserialize(deserializer, context)?;
        if let Some(max) = Self::max_len() {
            if vec.len() > max {
                return Err(D::Error::custom(format!(
                    "ProgressiveVariableList length {} exceeds maximum length {}",
                    vec.len(),
                    max
                )));
            }
        }
        Ok(ProgressiveVariableList::new(vec))
    }
}
