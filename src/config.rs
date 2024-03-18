use serde::de::Error;
use serde::ser::SerializeMap;
use std::path::PathBuf;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ManifestConfig {
    pub map: Vec<Mapping>,
}

#[derive(Debug)]
pub struct Mapping {
    pub from: PathBuf,
    pub to: PathBuf,
}

impl serde::Serialize for Mapping {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut serialize_map = serializer.serialize_map(Some(1))?;
        serialize_map.serialize_entry(&self.from, &self.to)?;
        serialize_map.end()
    }
}

impl<'de> serde::Deserialize<'de> for Mapping {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct MappingVisitor;

        impl<'de> serde::de::Visitor<'de> for MappingVisitor {
            type Value = Mapping;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a key value pair")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let (key, value) = map.next_entry()?.ok_or_else(|| {
                    A::Error::custom("expected exactly one key value pair, found none")
                })?;

                if map.next_key::<PathBuf>()?.is_some() {
                    return Err(A::Error::custom(
                        "expected exactly one one key value pair, found two or more",
                    ));
                }

                Ok(Mapping {
                    from: key,
                    to: value,
                })
            }
        }

        deserializer.deserialize_map(MappingVisitor)
    }
}
