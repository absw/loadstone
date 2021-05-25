use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum SecurityMode {
    Crc,
    P256ECDSA,
}

impl Default for SecurityMode {
    fn default() -> Self { SecurityMode::P256ECDSA }
}

#[derive(Default, Clone, Serialize)]
pub struct SecurityConfiguration {
    pub security_mode: SecurityMode,
    pub verifying_key_raw: String,
}
