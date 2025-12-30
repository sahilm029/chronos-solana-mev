mod types;
mod ingestor;
mod engine;
mod market_math;

use std::time::Instant;
use std::thread;
use std::sync::mpsc;
use ingestor::ProxyIngestor;
use engine::BlockBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "data/solana_trades.csv";
    if !std::path::Path::new(file_path).exists() {
         println!("Please run python gen_data.py first!");
         return Ok(());
    }

    println!("Starting Chronos Pipelined Engine...");
    println!("Architecture: Producer (Ingest) -> Channel -> Consumer (Engine)");
    
    let start = Instant::now();

    // 1. Create the Channel
    // We use a "SyncChannel" with a buffer of 10,000 trades.
    // If the Engine is slow, the Ingestor will pause (Backpressure).
    // This prevents RAM from exploding.
    let (tx, rx) = mpsc::sync_channel(10_000);

    // 2. Spawn Thread A: The Consumer (Engine)
    // We spawn this *first* so it's ready to receive.
    let engine_handle = thread::spawn(move || {
        let mut engine = BlockBuilder::new();
        
        // Loop until the channel is closed (when Sender drops)
        while let Ok(trade) = rx.recv() {
            engine.add_trade(trade);
        }
        
        // Finalize
        engine.flush();
        engine // Return the engine struct so we can read stats
    });

    // 3. Thread B: The Producer (Ingestor)
    // We run this on the main thread (or you could spawn another one)
    let mut ingestor = ProxyIngestor::new();
    let replay_hash = ingestor.process_file(file_path, tx)?; 
    // Note: 'tx' is dropped here automatically when function ends, 
    // closing the channel and telling Thread A to stop.

    // 4. Join the threads (Wait for Engine to finish)
    let final_engine = engine_handle.join().expect("Engine thread panicked");

    let duration = start.elapsed();
    
    println!("--------------------------------");
    println!("Status: Pipeline Complete");
    println!("Time: {:.2?}", duration);
    println!("--------------------------------");
    println!("RESULTS:");
    println!("Total Transactions:   {}", ingestor.records_processed);
    println!("Conflicted Txs:       {}", final_engine.conflicted_txs);
    println!("Total Value Lost:     ${:.2}", final_engine.total_regret_usd);
    println!("Replay Hash:          {}", replay_hash);
    println!("--------------------------------");

    Ok(())
}