// Re-exporting structures from waproto to avoid duplication
pub use waproto_ng::whatsapp::{
    IdentityKeyPairStructure, PreKeyRecordStructure, RecordStructure, SenderKeyRecordStructure,
    SenderKeyStateStructure, SessionStructure, SignedPreKeyRecordStructure,
};

pub use waproto_ng::whatsapp::sender_key_state_structure;
pub use waproto_ng::whatsapp::session_structure;
