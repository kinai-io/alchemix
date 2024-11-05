use serde::{de::DeserializeOwned, Serialize};


pub trait Entity: Serialize + DeserializeOwned + Clone {
    fn get_id(&self) -> &str;
    fn get_kind(&self) -> &str;
    fn get_key(&self) -> String;
    fn get_fields_index(&self) -> Vec<FieldIndex>;
}

#[derive(Debug)]
pub struct FieldIndex {
    pub kind: String,
    pub entity_id: String,
    pub name: String,
    pub value: String,
    pub stored_type: String
}