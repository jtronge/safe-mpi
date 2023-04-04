use serde::de::{Deserialize, Deserializer, Error, Visitor};
use std::convert::TryFrom;

#[derive(Clone, Debug)]
pub enum DataType {
    Simple,
    ComplexNoncompound,
    ComplexCompound,
}

const ALLOWED: &'static [&'static str] = &["simple", "complex-noncompound", "complex-compound"];

struct DataTypeVisitor;

impl<'de> Visitor<'de> for DataTypeVisitor {
    type Value = DataType;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "a string value from {:?}", ALLOWED)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match v {
            "simple" => Ok(DataType::Simple),
            "complex-noncompound" => Ok(DataType::ComplexNoncompound),
            "complex-compound" => Ok(DataType::ComplexCompound),
            _ => Err(E::unknown_variant(v, ALLOWED)),
        }
    }
}

impl<'de> Deserialize<'de> for DataType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(DataTypeVisitor)
    }
}

impl TryFrom<&str> for DataType {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "simple" => Ok(DataType::Simple),
            "complex-noncompound" => Ok(DataType::ComplexNoncompound),
            "complex-compound" => Ok(DataType::ComplexCompound),
            _ => Err("Invalid DataType literal"),
        }
    }
}
