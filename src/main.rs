mod types;
mod ingestor;
mod engine;

use std::time::Instant;
use ingestor::ProxyIngestor;
use engine::BlockBuilder;
use types::InternalTrade;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "data/solana_trades.csv";
    
    // Check for file existence (keeping your dummy check is fine)
    if !std::path::Path::new(file_path).exists() {
         println!("Please run python gen_data.py first!");
         return Ok(());
    }

    println!("Starting Chronos Simulation...");
    println!("Thesis: Measuring value lost to State Contention (Ordering Risks)");
    let start = Instant::now();
    
    let mut ingestor = ProxyIngestor::new();
    let mut engine = BlockBuilder::new();

    ingestor.process_file(file_path, |record| {
        let trade: InternalTrade = record.into();
        engine.add_trade(trade);
    })?;

    engine.flush(); // Process final block

    let duration = start.elapsed();
    
    println!("--------------------------------");
    println!("Status: Simulation Complete");
    println!("Time: {:.2?}", duration);
    println!("--------------------------------");
    println!("RESULTS:");
    println!("Total Transactions:   {}", ingestor.records_processed);
    println!("Conflicted Txs:       {} (Trades that got a worse price)", engine.conflicted_txs);
    println!("Total Value Lost:     ${:.2} (Simulated Regret)", engine.total_regret_usd);
    println!("--------------------------------");

    Ok(())
}