# Chronos: Deterministic MEV Execution Simulator

> "Most strategies fail not because the alpha is wrong, but because the execution is nondeterministic."

Chronos is a high-performance, offline execution engine written in **Rust**. It replays historical Solana transaction data to quantify **State Contention Risk**—the difference between *theoretical* backtest PnL and *realized* execution PnL under network congestion.

## ⚡ Performance Benchmark
- **Throughput:** ~1.2 Million Transactions / second (Single-threaded)
- **Latency:** <800ms for 1GB Dataset
- **Architecture:** Zero-Copy Deserialization (Serde + manual ByteRecord management)

## 🛠 Engineering Architecture

### 1. The Zero-Copy Ingestor
Uses `csv` with manual buffer management (`ByteRecord`) to strictly borrow `&str` from the raw OS buffer. This eliminates 90% of heap allocations during the parse phase.

### 2. The Block Builder (Engine)
Simulates a deterministic scheduler that mimics Solana's leader schedule behavior:
- **Ordering Pressure:** Re-sorts transactions based on `is_bundled` (Jito-Solana) and Priority Fees.
- **State Contention:** Uses a linear impact model to calculate slippage based on `preceding_slot_volume`.

### 3. Reproducibility
Every run emits a **Replay Hash (SHA-256)** derived from the input vector and seed. This guarantees that any debugging session is 100% reproducible, solving the "it worked on my machine" problem common in distributed systems.

## 📊 The "Strategy Killer" Experiment
Running a naive arbitrage strategy through Chronos revealed that **Ordering Latency** destroys alpha.

| Metric | Result |
|--------|--------|
| **Total Transactions** | 1,000,000 |
| **Conflicted Txs** | 900,003 (90%) |
| **Simulated Regret (Loss)** | **$626,678,837** |

*Conclusion: Being correct but second is the same as being wrong.*

## 🚀 Usage

```bash
# Generate 1GB of synthetic data
python3 gen_data.py

# Run the simulation (Release mode mandatory for vectorization)
cargo run --release