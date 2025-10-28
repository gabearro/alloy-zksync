use serde::{Deserialize, Serialize};
/// ZKsync transaction response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(
    into = "serde_from::TransactionEither",
    from = "serde_from::TransactionEither"
)]
pub struct TransactionResponse {
    #[serde(flatten)]
    inner: alloy::rpc::types::transaction::Transaction<crate::network::tx_envelope::TxEnvelope>,
}

impl alloy::consensus::Transaction for TransactionResponse {
    fn chain_id(&self) -> Option<alloy::primitives::ChainId> {
        self.inner.chain_id()
    }

    fn nonce(&self) -> u64 {
        self.inner.nonce()
    }

    fn gas_limit(&self) -> u64 {
        self.inner.gas_limit()
    }

    fn gas_price(&self) -> Option<u128> {
        self.inner.gas_price()
    }

    fn max_fee_per_gas(&self) -> u128 {
        self.inner.max_fee_per_gas()
    }

    fn max_priority_fee_per_gas(&self) -> Option<u128> {
        self.inner.max_priority_fee_per_gas()
    }

    fn max_fee_per_blob_gas(&self) -> Option<u128> {
        self.inner.max_fee_per_blob_gas()
    }

    fn priority_fee_or_price(&self) -> u128 {
        self.inner.priority_fee_or_price()
    }

    fn to(&self) -> Option<alloy::primitives::Address> {
        self.inner.to()
    }

    fn is_create(&self) -> bool {
        self.inner.is_create()
    }

    fn value(&self) -> alloy::primitives::U256 {
        self.inner.value()
    }

    fn input(&self) -> &alloy::primitives::Bytes {
        self.inner.input()
    }

    fn access_list(&self) -> Option<&alloy::rpc::types::AccessList> {
        self.inner.access_list()
    }

    fn blob_versioned_hashes(&self) -> Option<&[alloy::primitives::B256]> {
        self.inner.blob_versioned_hashes()
    }

    fn authorization_list(&self) -> Option<&[alloy::eips::eip7702::SignedAuthorization]> {
        self.inner.authorization_list()
    }

    fn kind(&self) -> alloy::primitives::TxKind {
        self.inner.kind()
    }

    fn effective_gas_price(&self, base_fee: Option<u64>) -> u128 {
        self.inner.effective_gas_price(base_fee)
    }

    fn is_dynamic_fee(&self) -> bool {
        self.inner.is_dynamic_fee()
    }
}

impl alloy::consensus::Typed2718 for TransactionResponse {
    fn ty(&self) -> u8 {
        self.inner.ty()
    }
}

impl alloy::network::TransactionResponse for TransactionResponse {
    fn tx_hash(&self) -> alloy::primitives::TxHash {
        self.inner.tx_hash()
    }

    fn from(&self) -> alloy::primitives::Address {
        self.inner.from()
    }

    fn block_hash(&self) -> Option<alloy::primitives::BlockHash> {
        self.inner.block_hash()
    }

    fn block_number(&self) -> Option<u64> {
        self.inner.block_number()
    }

    fn transaction_index(&self) -> Option<u64> {
        self.inner.transaction_index()
    }
}

impl AsRef<crate::network::tx_envelope::TxEnvelope> for TransactionResponse {
    fn as_ref(&self) -> &crate::network::tx_envelope::TxEnvelope {
        &self.inner.inner
    }
}

mod serde_from {
    //! NB: Why do we need this?
    //!
    //! Helper module for serializing and deserializing ZKsync [`TransactionResponse`].
    //!
    //! This is needed because we might need to deserialize the `from` field into both
    //! [`field@alloy::rpc::types::transaction::Transaction::from`] and [`field@TxEip712::from`].
    use super::TransactionResponse;
    use crate::network::tx_envelope::TxEnvelope;
    use crate::network::unsigned_tx::eip712::TxEip712;
    use alloy::consensus::{Signed, transaction::Recovered};
    use alloy::primitives::{BlockHash, Signature, U256};
    use serde::{Deserialize, Serialize};

    /// Exactly the same thing as [`alloy::rpc::types::transaction::Transaction`] but without the
    /// `from` field. We need it because [`TxEnvelope::Eip712`] can consume `from` first thus
    /// failing the entire deserialization process.
    #[derive(Serialize, Deserialize)]
    pub struct TransactionWithoutFrom {
        #[serde(flatten)]
        pub inner: Signed<TxEip712>,
        #[serde(default, rename = "blockHash")]
        pub block_hash: Option<BlockHash>,
        // JSON-RPC uses 0x-prefixed hex strings for numeric fields. We deserialize into String
        // here and parse manually to avoid relying on alloy's private serde helpers.
        #[serde(default, rename = "blockNumber")]
        pub block_number: Option<String>,
        #[serde(default, rename = "transactionIndex")]
        pub transaction_index: Option<String>,
        #[serde(default, rename = "effectiveGasPrice")]
        pub effective_gas_price: Option<String>,
    }

    /// Same as TransactionWithoutFrom but without signature fields, since some nodes
    /// omit them in full tx objects. We'll synthesize a dummy signature during conversion.
    #[derive(Serialize, Deserialize)]
    pub struct TransactionWithoutFromUnsigned {
        #[serde(flatten)]
        pub inner: TxEip712,
        #[serde(default, rename = "blockHash")]
        pub block_hash: Option<BlockHash>,
        #[serde(default, rename = "blockNumber")]
        pub block_number: Option<String>,
        #[serde(default, rename = "transactionIndex")]
        pub transaction_index: Option<String>,
        #[serde(default, rename = "effectiveGasPrice")]
        pub effective_gas_price: Option<String>,
    }

    /// (De)serializes both regular [`alloy::rpc::types::transaction::Transaction`] and [`TransactionWithoutFrom`].
    #[derive(Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum TransactionEither {
        Regular(alloy::rpc::types::transaction::Transaction<TxEnvelope>),
        WithoutFrom(TransactionWithoutFrom),
        // Some zkSync nodes omit signature fields in full tx objects for type 0x71.
        // Accept an unsigned EIP-712 tx and synthesize a zero signature.
        WithoutFromUnsigned(TransactionWithoutFromUnsigned),
        // Standard Ethereum transactions (Legacy, EIP-2930, EIP-1559, EIP-4844)
        // that might have missing optional fields
        StandardEthereum(StandardEthereumTransaction),
    }

    /// A more lenient deserialization of standard Ethereum transactions that accepts
    /// missing optional fields (like accessList, blobVersionedHashes, etc.)
    #[derive(Serialize, Deserialize)]
    pub struct StandardEthereumTransaction {
        #[serde(flatten)]
        pub inner: LenientTxEnvelope,
        pub from: alloy::primitives::Address,
        #[serde(default, rename = "blockHash")]
        pub block_hash: Option<BlockHash>,
        #[serde(default, rename = "blockNumber")]
        pub block_number: Option<String>,
        #[serde(default, rename = "transactionIndex")]
        pub transaction_index: Option<String>,
        #[serde(default, rename = "effectiveGasPrice")]
        pub effective_gas_price: Option<String>,
    }

    /// A lenient version of transaction envelope that accepts missing optional fields
    #[derive(Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum LenientTxEnvelope {
        /// Legacy transaction
        Legacy(LenientSignedLegacyTx),
        /// EIP-2930 transaction
        Eip2930(LenientSignedEip2930Tx),
        /// EIP-1559 transaction
        Eip1559(LenientSignedEip1559Tx),
        /// EIP-4844 transaction
        Eip4844(LenientSignedEip4844Tx),
    }

    #[derive(Serialize, Deserialize)]
    pub struct LenientSignedLegacyTx {
        #[serde(flatten)]
        pub tx: alloy::consensus::TxLegacy,
        pub r: alloy::primitives::U256,
        pub s: alloy::primitives::U256,
        #[serde(default)]
        pub v: alloy::primitives::U256,
    }

    #[derive(Serialize, Deserialize)]
    pub struct LenientSignedEip2930Tx {
        #[serde(flatten)]
        pub tx: alloy::consensus::TxEip2930,
        pub r: alloy::primitives::U256,
        pub s: alloy::primitives::U256,
        #[serde(default, rename = "yParity")]
        pub y_parity: Option<String>,
        #[serde(default)]
        pub v: Option<String>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct LenientSignedEip1559Tx {
        #[serde(flatten)]
        pub tx: alloy::consensus::TxEip1559,
        pub r: alloy::primitives::U256,
        pub s: alloy::primitives::U256,
        #[serde(default, rename = "yParity")]
        pub y_parity: Option<String>,
        #[serde(default)]
        pub v: Option<String>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct LenientSignedEip4844Tx {
        #[serde(flatten)]
        pub tx: alloy::consensus::TxEip4844,
        #[serde(default, rename = "blobVersionedHashes")]
        pub blob_versioned_hashes: Option<Vec<alloy::primitives::B256>>,
        pub r: alloy::primitives::U256,
        pub s: alloy::primitives::U256,
        #[serde(default, rename = "yParity")]
        pub y_parity: Option<String>,
        #[serde(default)]
        pub v: Option<String>,
    }

    impl From<TransactionEither> for TransactionResponse {
        fn from(value: TransactionEither) -> Self {
            fn parse_u64_opt_hex(s: &Option<String>) -> Option<u64> {
                s.as_ref().and_then(|v| {
                    let v = v.trim();
                    if v.is_empty() { return None; }
                    let v = v.strip_prefix("0x").unwrap_or(v);
                    u64::from_str_radix(v, 16).ok()
                })
            }
            fn parse_u128_opt_hex(s: &Option<String>) -> Option<u128> {
                s.as_ref().and_then(|v| {
                    let v = v.trim();
                    if v.is_empty() { return None; }
                    let v = v.strip_prefix("0x").unwrap_or(v);
                    u128::from_str_radix(v, 16).ok()
                })
            }

            fn parse_parity(y_parity: &Option<String>, v: &Option<String>) -> bool {
                // Try yParity first
                if let Some(yp) = y_parity {
                    let yp = yp.trim().strip_prefix("0x").unwrap_or(yp);
                    return yp == "1" || yp == "01";
                }
                // Fall back to v
                if let Some(v_str) = v {
                    let v_str = v_str.trim().strip_prefix("0x").unwrap_or(v_str);
                    if let Ok(v_val) = u64::from_str_radix(v_str, 16) {
                        return v_val % 2 == 0;
                    }
                }
                false
            }

            match value {
                TransactionEither::Regular(tx) => TransactionResponse { inner: tx },
                TransactionEither::WithoutFrom(value) => {
                    let from = value.inner.tx().from;
                    TransactionResponse {
                        inner: alloy::rpc::types::transaction::Transaction {
                            inner: Recovered::new_unchecked(TxEnvelope::Eip712(value.inner), from),
                            block_hash: value.block_hash,
                            block_number: parse_u64_opt_hex(&value.block_number),
                            transaction_index: parse_u64_opt_hex(&value.transaction_index),
                            effective_gas_price: parse_u128_opt_hex(&value.effective_gas_price),
                        },
                    }
                }
                TransactionEither::WithoutFromUnsigned(value) => {
                    let from = value.inner.from;
                    // Synthesize a zero signature; hash will be computed lazily if needed.
                    let dummy_sig = Signature::new(U256::ZERO, U256::ZERO, false);
                    let signed = Signed::new_unhashed(value.inner, dummy_sig);
                    TransactionResponse {
                        inner: alloy::rpc::types::transaction::Transaction {
                            inner: Recovered::new_unchecked(TxEnvelope::Eip712(signed), from),
                            block_hash: value.block_hash,
                            block_number: parse_u64_opt_hex(&value.block_number),
                            transaction_index: parse_u64_opt_hex(&value.transaction_index),
                            effective_gas_price: parse_u128_opt_hex(&value.effective_gas_price),
                        },
                    }
                }
                TransactionEither::StandardEthereum(value) => {
                    let envelope = match value.inner {
                        LenientTxEnvelope::Legacy(tx) => {
                            let sig = Signature::from_scalars_and_parity(tx.r.into(), tx.s.into(), tx.v != U256::ZERO);
                            let signed = Signed::new_unchecked(tx.tx, sig, Default::default());
                            alloy::consensus::TxEnvelope::Legacy(signed)
                        }
                        LenientTxEnvelope::Eip2930(tx) => {
                            let parity = parse_parity(&tx.y_parity, &tx.v);
                            let sig = Signature::from_scalars_and_parity(tx.r.into(), tx.s.into(), parity);
                            let signed = Signed::new_unchecked(tx.tx, sig, Default::default());
                            alloy::consensus::TxEnvelope::Eip2930(signed)
                        }
                        LenientTxEnvelope::Eip1559(tx) => {
                            let parity = parse_parity(&tx.y_parity, &tx.v);
                            let sig = Signature::from_scalars_and_parity(tx.r.into(), tx.s.into(), parity);
                            let signed = Signed::new_unchecked(tx.tx, sig, Default::default());
                            alloy::consensus::TxEnvelope::Eip1559(signed)
                        }
                        LenientTxEnvelope::Eip4844(tx) => {
                            let parity = parse_parity(&tx.y_parity, &tx.v);
                            let sig = Signature::from_scalars_and_parity(tx.r.into(), tx.s.into(), parity);
                            let signed = Signed::new_unchecked(tx.tx, sig, Default::default());
                            alloy::consensus::TxEnvelope::Eip4844(signed.into())
                        }
                    };

                    TransactionResponse {
                        inner: alloy::rpc::types::transaction::Transaction {
                            inner: Recovered::new_unchecked(TxEnvelope::Native(envelope), value.from),
                            block_hash: value.block_hash,
                            block_number: parse_u64_opt_hex(&value.block_number),
                            transaction_index: parse_u64_opt_hex(&value.transaction_index),
                            effective_gas_price: parse_u128_opt_hex(&value.effective_gas_price),
                        },
                    }
                }
            }
        }
    }

    impl From<TransactionResponse> for TransactionEither {
        fn from(value: TransactionResponse) -> Self {
            fn to_hex_opt_u64(v: &Option<u64>) -> Option<String> {
                v.map(|x| format!("0x{:x}", x))
            }
            fn to_hex_opt_u128(v: &Option<u128>) -> Option<String> {
                v.map(|x| format!("0x{:x}", x))
            }
            match value.inner.inner.as_ref() {
                TxEnvelope::Native(_) => TransactionEither::Regular(value.inner),
                TxEnvelope::Eip712(signed) => {
                    TransactionEither::WithoutFrom(TransactionWithoutFrom {
                        inner: signed.clone(),
                        block_hash: value.inner.block_hash,
                        block_number: to_hex_opt_u64(&value.inner.block_number),
                        transaction_index: to_hex_opt_u64(&value.inner.transaction_index),
                        effective_gas_price: to_hex_opt_u128(&value.inner.effective_gas_price),
                    })
                }
            }
        }
    }
}