// SPDX-License-Identifier: Apache-2.0
//
// Copyright © 2017 Trust Wallet.

//! Original source code: https://github.com/solana-labs/solana/blob/4b65cc8eef6ef79cb9b9cbc534a99b4900e58cf7/sdk/program/src/message/compiled_keys.rs

use crate::address::SolanaAddress;
use crate::instruction::Instruction;
use crate::transaction::MessageHeader;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use tw_coin_entry::error::{SigningError, SigningErrorType, SigningResult};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
struct CompiledKeyMeta {
    is_signer: bool,
    is_writable: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub(crate) struct CompiledKeys {
    ordered_keys: Vec<SolanaAddress>,
    key_meta_map: HashMap<SolanaAddress, CompiledKeyMeta>,
}

impl CompiledKeys {
    pub fn compile(instructions: &[Instruction]) -> Self {
        let mut key_meta_map = HashMap::<SolanaAddress, CompiledKeyMeta>::new();
        let mut ordered_keys = Vec::default();

        for ix in instructions {
            for account_meta in &ix.accounts {
                let meta_entry = key_meta_map.entry(account_meta.pubkey);
                if matches!(meta_entry, Entry::Vacant(_)) {
                    ordered_keys.push(account_meta.pubkey);
                }

                let meta = meta_entry.or_default();
                meta.is_signer |= account_meta.is_signer;
                meta.is_writable |= account_meta.is_writable;
            }
        }

        // add programIds (read-only, at end)
        for ix in instructions {
            key_meta_map.entry(ix.program_id).or_default();
            ordered_keys.push(ix.program_id);
        }

        Self {
            ordered_keys,
            key_meta_map,
        }
    }

    pub(crate) fn try_into_message_components(
        self,
    ) -> SigningResult<(MessageHeader, Vec<SolanaAddress>)> {
        let try_into_u8 = |num: usize| -> SigningResult<u8> {
            u8::try_from(num).map_err(|_| SigningError(SigningErrorType::Error_tx_too_big))
        };

        let Self {
            ordered_keys,
            key_meta_map,
        } = self;

        let filter = |account, is_signer: bool, is_writable: bool| -> Option<SolanaAddress> {
            let meta = key_meta_map.get(account).copied().unwrap_or_default();
            (meta.is_signer == is_signer && meta.is_writable == is_writable).then_some(*account)
        };

        let writable_signer_keys: Vec<SolanaAddress> = ordered_keys
            .iter()
            .filter_map(|key| filter(key, true, true))
            .collect();
        let readonly_signer_keys: Vec<SolanaAddress> = ordered_keys
            .iter()
            .filter_map(|key| filter(key, true, false))
            .collect();
        let writable_non_signer_keys: Vec<SolanaAddress> = ordered_keys
            .iter()
            .filter_map(|key| filter(key, false, true))
            .collect();
        let readonly_non_signer_keys: Vec<SolanaAddress> = ordered_keys
            .iter()
            .filter_map(|key| filter(key, false, false))
            .collect();

        let signers_len = writable_signer_keys
            .len()
            .saturating_add(readonly_signer_keys.len());

        let header = MessageHeader {
            num_required_signatures: try_into_u8(signers_len)?,
            num_readonly_signed_accounts: try_into_u8(readonly_signer_keys.len())?,
            num_readonly_unsigned_accounts: try_into_u8(readonly_non_signer_keys.len())?,
        };

        let static_account_keys = std::iter::empty()
            .chain(writable_signer_keys)
            .chain(readonly_signer_keys)
            .chain(writable_non_signer_keys)
            .chain(readonly_non_signer_keys)
            .collect();

        Ok((header, static_account_keys))
    }
}
