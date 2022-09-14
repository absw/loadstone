use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityMode {
    /// Enforces image integrity through a cyclical redundancy check.
    /// This only helps against unintentional corruption, and doesn't
    /// protect against any kind of attack.
    Crc,
    /// Enforces P256 ECDSA signature verification. This ensures integrity
    /// and authenticity, but not secrecy (image is not encrypted).
    P256ECDSA,
}

impl Default for SecurityMode {
    fn default() -> Self {
        SecurityMode::P256ECDSA
    }
}

/// Defines how Loadstone will aproach guaranteeing image security
/// (integrity, secrecy and authenticity).
#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct SecurityConfiguration {
    pub security_mode: SecurityMode,
    /// String format (PEM) of the verifying public key.
    pub verifying_key_raw: String,
}
