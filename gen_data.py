import csv
import random
import time
import hashlib

# Configuration
NUM_ROWS = 1_000_000  # 1 Million trades
OUTPUT_FILE = "data/solana_trades.csv"

print(f"Generating {NUM_ROWS} trades...")
start = time.time()

with open(OUTPUT_FILE, "w", newline="") as f:
    writer = csv.writer(f)
    # Header must match your Rust struct fields exactly
    writer.writerow([
        "slot", "timestamp", "amount_in", "amount_out", 
        "tx_signature", "token_mint_in", "token_mint_out", 
        "is_bundled", "tx_index"
    ])
    
    # Pre-generate some static data to speed up Python (we want to test Rust, not Python)
    mints = [hashlib.md5(str(i).encode()).hexdigest() for i in range(100)]
    
    for i in range(NUM_ROWS):
        slot = 200_000_000 + (i // 1000) # New slot every 1000 txs
        writer.writerow([
            slot,
            1700000000 + i, # timestamp
            random.randint(1000, 1000000), # amount_in
            random.randint(1000, 1000000), # amount_out
            hashlib.sha256(str(i).encode()).hexdigest()[:44], # Fake solana signature
            random.choice(mints), # mint_in
            random.choice(mints), # mint_out
            "true" if random.random() < 0.1 else "false", # is_bundled
            i % 2000 # tx_index
        ])
        
        if i % 100_000 == 0:
            print(f"Generated {i} rows...")

end = time.time()
print(f"Done! Created {OUTPUT_FILE} in {end - start:.2f}s")