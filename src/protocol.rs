use std::{fs::read_to_string, path::Path};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub r#type: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Object {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Protocol {
    pub models: Vec<Object>,
    pub requests: Vec<Object>,
    pub updates: Vec<Object>,
}

impl Protocol {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let str = read_to_string(path)?;
        let mut protocol: Self = toml::from_str(&str)?;

        Self::add_prefix("rq_", &mut protocol.requests);
        Self::add_prefix("up_", &mut protocol.updates);

        Ok(protocol)
    }

    fn add_prefix(prefix: &str, objects: &mut [Object]) {
        objects
            .iter_mut()
            .for_each(|request| request.name.insert_str(0, prefix));
    }
}
