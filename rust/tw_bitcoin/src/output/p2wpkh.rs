use crate::{Error, Recipient, Result};
use bitcoin::{ScriptBuf, WPubkeyHash};

#[derive(Debug, Clone)]
pub struct TxOutputP2WPKH {
    pub(super) satoshis: u64,
    pub(super) script_pubkey: ScriptBuf,
}

impl TxOutputP2WPKH {
    pub fn new(satoshis: u64, recipient: Recipient<WPubkeyHash>) -> Self {
        TxOutputP2WPKH {
            satoshis,
            script_pubkey: ScriptBuf::new_v0_p2wpkh(recipient.wpubkey_hash()),
        }
    }
    pub fn from_bytes(bytes: Vec<u8>, satoshis: u64) -> Self {
        let script_pubkey = ScriptBuf::from_bytes(bytes);

        TxOutputP2WPKH {
            satoshis,
            script_pubkey,
        }
    }
    pub fn builder() -> TxOutputP2WPKHBuilder {
        TxOutputP2WPKHBuilder::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct TxOutputP2WPKHBuilder {
    satoshis: Option<u64>,
    recipient: Option<Recipient<WPubkeyHash>>,
}

impl TxOutputP2WPKHBuilder {
    pub fn new() -> TxOutputP2WPKHBuilder {
        Self::default()
    }
    pub fn satoshis(mut self, satoshis: u64) -> TxOutputP2WPKHBuilder {
        self.satoshis = Some(satoshis);
        self
    }
    pub fn recipient(mut self, recipient: Recipient<WPubkeyHash>) -> TxOutputP2WPKHBuilder {
        self.recipient = Some(recipient);
        self
    }
    pub fn build(self) -> Result<TxOutputP2WPKH> {
        Ok(TxOutputP2WPKH::new(
            self.satoshis.ok_or(Error::Todo)?,
            self.recipient.ok_or(Error::Todo)?,
        ))
    }
}