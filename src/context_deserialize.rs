use crate::{typenum::Unsigned, FixedVector, ProgressiveVariableList};
use context_deserialize::ContextDeserialize;
use serde::de::{DeserializeSeed, Deserializer, Error, SeqAccess, Visitor};
use std::marker::PhantomData;

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
        deserializer.deserialize_seq(ProgressiveListVisitor {
            context,
            _phantom: PhantomData,
        })
    }
}

/// Pushes one element at a time, so an oversized input fails at the first extra element.
struct ProgressiveListVisitor<C, T, N> {
    context: C,
    _phantom: PhantomData<(T, N)>,
}

impl<'de, C, T, N> Visitor<'de> for ProgressiveListVisitor<C, T, N>
where
    T: ContextDeserialize<'de, C>,
    N: Unsigned,
    C: Clone,
{
    type Value = ProgressiveVariableList<T, N>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a list")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut list = ProgressiveVariableList::empty();
        while let Some(value) = seq.next_element_seed(ElementSeed {
            context: self.context.clone(),
            _phantom: PhantomData,
        })? {
            list.push(value).map_err(A::Error::custom)?;
        }
        Ok(list)
    }
}

struct ElementSeed<C, T> {
    context: C,
    _phantom: PhantomData<T>,
}

impl<'de, C, T> DeserializeSeed<'de> for ElementSeed<C, T>
where
    T: ContextDeserialize<'de, C>,
{
    type Value = T;

    fn deserialize<D>(self, deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::context_deserialize(deserializer, self.context)
    }
}

#[cfg(test)]
mod test {
    use crate::ProgressiveVariableList;
    use context_deserialize::ContextDeserialize;
    use typenum::U4;

    fn context_deserialize_u64s(json: &str) -> Result<ProgressiveVariableList<u64, U4>, String> {
        let mut deserializer = serde_json::Deserializer::from_str(json);
        ProgressiveVariableList::context_deserialize(&mut deserializer, ())
            .map_err(|e| e.to_string())
    }

    #[test]
    fn context_deserialize_accepts_list_at_limit() {
        let list = context_deserialize_u64s("[1,2,3,4]").unwrap();
        assert_eq!(&list[..], &[1, 2, 3, 4]);
    }

    #[test]
    fn context_deserialize_fails_at_first_item_past_limit() {
        // The item after the limit is not a number. Only an early check reports the limit.
        let err = context_deserialize_u64s(r#"[1,2,3,4,5,"not a number"]"#).unwrap_err();
        assert!(
            err.contains("Index out of bounds: index 5, length 4"),
            "{err}"
        );
    }
}
