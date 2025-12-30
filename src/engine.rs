use crate::types::InternalTrade;
use crate::market_math::ConstantProductPool; // Import our new math kernel
use std::cmp::Ordering;
use std::collections::HashMap;

pub struct BlockBuilder {
    current_slot: u64,
    buffer: Vec<InternalTrade>,
    // CTO LEVEL UPDATE: We store stateful pools now
    // Map<TokenMint, PoolState>
    pools: HashMap<String, ConstantProductPool>, 
    
    // Metrics
    pub total_regret_usd: f64, 
    pub conflicted_txs: u64,
}

impl BlockBuilder {
    pub fn new() -> Self {
        Self {
            current_slot: 0,
            buffer: Vec::with_capacity(5000),
            pools: HashMap::new(),
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
        // Jito Bundles -> Priority Fee -> Normal
        self.buffer.sort_by(|a, b| {
            if a.is_bundled && !b.is_bundled { return Ordering::Less; }
            if !a.is_bundled && b.is_bundled { return Ordering::Greater; }
            a.tx_index.cmp(&b.tx_index)
        });

        // 2. STATE CONTENTION (Execution Force: AMM Physics)
        for trade in &self.buffer {
            // A. Get or Initialize the Pool for this Token
            // Assumption: 1M tokens vs 1M USDC starting liquidity (1:1 Price)
            let pool = self.pools.entry(trade.token_mint_in.clone())
                .or_insert_with(|| ConstantProductPool::new(1_000_000_000, 1_000_000_000));
            
            // B. Snapshot the price BEFORE this trade executes (The "Best" price)
            let theoretical_price = pool.get_spot_price();

            // C. Execute the swap against the AMM State
            // This actually changes the reserves (slippage occurs here)
            let actual_amount_out = pool.swap_base_for_quote(trade.amount_in);

            // D. Calculate Regret
            // "Theoretical Amount" = AmountIn * SpotPrice
            let theoretical_amount_out = (trade.amount_in as f64) * theoretical_price;
            let realized_amount_out = actual_amount_out as f64;

            // Regret = How much did I lose because the pool moved against me?
            // If I was first, I would have gotten theoretical_amount_out.
            let regret = theoretical_amount_out - realized_amount_out;

            if regret > 0.0 {
                self.total_regret_usd += regret;
                self.conflicted_txs += 1;
            }
        }

        // RESET POOLS for the next slot? 
        // In a real blockchain, pools persist. In this simulation, to keep it simple 
        // and avoid draining liquidity to zero over 1M trades, we might want to reset.
        // For now, let's KEEP them persistent to see "Liquidity Drain" effects.
        
        self.buffer.clear();
    }
}