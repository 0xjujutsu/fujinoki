pub mod title;

#[macro_export]
macro_rules! impl_deserialize_from_bits {
    ($name:ident, $type:ty) => {
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = <$type>::deserialize(_deserializer)?;
                Ok($name::from_bits_truncate(value))
            }
        }
    };
}
