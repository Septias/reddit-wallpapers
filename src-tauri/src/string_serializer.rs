use core::fmt::Debug;
use serde::ser::{Serialize, Serializer};
pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Debug,
    S: Serializer,
{
    let j = format!("{:?}", value);
    j.serialize(serializer)
}
