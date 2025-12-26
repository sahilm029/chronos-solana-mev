use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RawTradeRecord<'a> {
    pub slot: u64,
    pub timestamp: u64,
    pub amount_in: u64,
    pub amount_out: u64,
    pub tx_signature: &'a str,
    pub token_mint_in: &'a str,   // We need this now
    pub token_mint_out: &'a str,
    pub is_bundled: bool,
    pub tx_index: u16, 
}

#[derive(Debug, Clone)]
pub struct InternalTrade {
    pub slot: u64,
    pub amount_in: u64,
    pub is_bundled: bool,
    pub tx_index: u16, 
    pub tx_signature: String,
    // NEW: We track the token to find conflicts
    pub token_mint_in: String, 
}

impl From<RawTradeRecord<'_>> for InternalTrade {
    fn from(raw: RawTradeRecord<'_>) -> Self {
        Self {
            slot: raw.slot,
            amount_in: raw.amount_in,
            is_bundled: raw.is_bundled,
            tx_index: raw.tx_index,
            tx_signature: raw.tx_signature.to_string(),
            token_mint_in: raw.token_mint_in.to_string(), // Allocation
        }
    }
}