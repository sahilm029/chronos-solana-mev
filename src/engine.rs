use crate::types::InternalTrade;
use std::cmp::Ordering;
use std::collections::HashMap;

pub struct BlockBuilder {
    current_slot: u64,
    buffer: Vec<InternalTrade>,
    // Metrics
    pub total_regret_usd: f64, 
    pub conflicted_txs: u64,
}

impl BlockBuilder {
    pub fn new() -> Self {
        Self {
            current_slot: 0,
            buffer: Vec::with_capacity(5000),
            total_regret_usd: 0.0,
            conflicted_txs: 0,
        }
    }

    pub fn add_trade(&mut self, trade: InternalTrade) {
        if self.buffer.is_empty() {
            self.current_slot = trade.slot;
        }

        if trade.slot != self.current_slot {
            self.process_block();
            self.current_slot = trade.slot;
        }

        self.buffer.push(trade);
    }

    pub fn flush(&mut self) {
        self.process_block();
    }

    fn process_block(&mut self) {
        if self.buffer.is_empty() { return; }

        // 1. SORTING (Execution Force: Ordering Pressure)
        // Jito Bundles (true) -> Priority Fee (simulated by index) -> Normal
        self.buffer.sort_by(|a, b| {
            if a.is_bundled && !b.is_bundled { return Ordering::Less; }
            if !a.is_bundled && b.is_bundled { return Ordering::Greater; }
            a.tx_index.cmp(&b.tx_index)
        });

        // 2. STATE CONTENTION (Execution Force: Impact)
        // We track "Volume Seen So Far" for each token in this specific slot.
        let mut slot_volume: HashMap<String, u64> = HashMap::new();

        for trade in &self.buffer {
            // Get current volume BEFORE this trade executes
            let volume_before = *slot_volume.get(&trade.token_mint_in).unwrap_or(&0);

            // SIMULATION: Linear Price Impact
            // For every 100,000 units traded before you, price worsens by $0.05
            // Regret = (Price Impact) * Your Amount
            if volume_before > 0 {
                let slippage_per_unit = (volume_before as f64 / 100_000.0) * 0.05;
                let regret = slippage_per_unit * (trade.amount_in as f64 / 1_000.0); // Simple scaler
                
                self.total_regret_usd += regret;
                self.conflicted_txs += 1;
            }

            // Update volume for the NEXT person in line
            slot_volume.insert(
                trade.token_mint_in.clone(), 
                volume_before + trade.amount_in
            );
        }

        self.buffer.clear();
    }
}