use serde::{Deserialize, Serialize};
use bitflags::bitflags;

bitflags! {
    pub struct ResourceFlags: u32 {
        const Completed  = 0b00000001;
    }
}

impl Serialize for ResourceFlags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut flags = Vec::new();
        if self.contains(ResourceFlags::Completed) {
            flags.push("Completed");
        }
        flags.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ResourceFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let flags: Vec<String> = Vec::deserialize(deserializer)?;
        let mut result = ResourceFlags::empty();
        for flag in flags {
            match flag.as_str() {
                "Completed" => result.insert(ResourceFlags::Completed),
                _ => {}
            }
        }
        Ok(result)
    }
}

#[derive(Serialize)]
pub struct Resource {
    pub id: i64,
    pub name: String,
    pub flags: ResourceFlags,
    pub group_ids: Vec<i64>,
    pub created: String,
    pub unique_global_id: String,
}