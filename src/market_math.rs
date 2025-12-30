/// Represents a Liquidity Pool (AMM) adhering to x * y = k
/// We assume a standard 50/50 pool (like Raydium/Uniswap V2).
#[derive(Debug, Clone)]
pub struct ConstantProductPool {
    pub reserve_base: u64,  // Token X (e.g., SOL)
    pub reserve_quote: u64, // Token Y (e.g., USDC)
    pub k: u128,            // The invariant (x * y)
}

impl ConstantProductPool {
    /// Initialize a new pool with starting liquidity
    pub fn new(reserve_base: u64, reserve_quote: u64) -> Self {
        let k = (reserve_base as u128) * (reserve_quote as u128);
        Self {
            reserve_base,
            reserve_quote,
            k,
        }
    }

    /// Calculate how much 'Quote' token you get for selling 'Base' token.
    /// This effectively calculates the price impact of the trade.
    /// Formula: dy = y - (k / (x + dx))
    pub fn swap_base_for_quote(&mut self, amount_in: u64) -> u64 {
        // 1. Calculate new reserve X (base + input)
        let new_reserve_base = self.reserve_base + amount_in;

        // 2. Calculate new reserve Y (k / new_x)
        // We use integer division, which naturally floors (standard DeFi behavior)
        let new_reserve_quote = (self.k / new_reserve_base as u128) as u64;

        // 3. The amount out is the difference (Old Y - New Y)
        let amount_out = self.reserve_quote - new_reserve_quote;

        // 4. Update the state (Actually move the liquidity)
        self.reserve_base = new_reserve_base;
        self.reserve_quote = new_reserve_quote;

        amount_out
    }

    /// Get the current "Spot Price" (Price for 1 tiny unit)
    pub fn get_spot_price(&self) -> f64 {
        self.reserve_quote as f64 / self.reserve_base as f64
    }
}