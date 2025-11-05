use rand::Rng;

use crate::model::{Model, ModelGroupMeta, User};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ActionType {
    Create = 0,
    Update = 1,
    Delete = 2,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum EntityType {
    Model = 0,
    Resource = 1,
    Label = 2,
    Group = 3,
}

pub struct AuditEntry {
    pub id: String,
    pub user_id: i64,
    pub action_type: ActionType,
    pub entity_type: EntityType,
    pub global_entity_id: String,
    pub timestamp: String,
}

pub fn random_hex_32() -> String {
    let mut bytes = [0u8; 16];
    rand::rng().fill(&mut bytes);
    hex::encode(bytes)
}

pub fn time_now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

impl AuditEntry {
    pub fn new(user : &User, action_type : ActionType, entity_type : EntityType, global_entity_id : String) -> AuditEntry
    {
        AuditEntry {
            id: random_hex_32(),
            user_id: user.id,
            action_type: action_type,
            entity_type: entity_type,
            global_entity_id: global_entity_id,
            timestamp: "".into()
        }
    }

    pub fn from_model(user : &User, action_type : ActionType, model : Model) -> AuditEntry
    {
        AuditEntry::new(user, action_type, EntityType::Model, model.unique_global_id)
    }

    pub fn from_group(user : &User, action_type : ActionType, group : ModelGroupMeta) -> AuditEntry
    {
        AuditEntry::new(user, action_type, EntityType::Group, group.unique_global_id)
    }

    pub fn from_label(user : &User, action_type : ActionType, label_global_id : String) -> AuditEntry
    {
        AuditEntry::new(user, action_type, EntityType::Label, label_global_id)
    }

    pub fn from_resource(user : &User, action_type : ActionType, resource_global_id : String) -> AuditEntry
    {
        AuditEntry::new(user, action_type, EntityType::Resource, resource_global_id)
    }
}