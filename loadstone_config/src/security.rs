use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum SecurityMode {
    Crc,
    P256ECDSA,
}

impl Default for SecurityMode {
    fn default() -> Self { SecurityMode::P256ECDSA }
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct SecurityConfiguration {
    pub security_mode: SecurityMode,
    pub verifying_key_raw: String,
}
