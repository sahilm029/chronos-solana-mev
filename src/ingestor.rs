use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use sha2::{Sha256, Digest};
use csv::ByteRecord;
use crate::types::{RawTradeRecord, InternalTrade};
use std::sync::mpsc::SyncSender; // Bounded Channel

#[derive(thiserror::Error, Debug)]
pub enum IngestError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("CSV Parsing Error: {0}")]
    Csv(#[from] csv::Error),
}

pub struct ProxyIngestor {
    replay_hasher: Sha256,
    pub records_processed: u64,
}

impl ProxyIngestor {
    pub fn new() -> Self {
        Self {
            replay_hasher: Sha256::new(),
            records_processed: 0,
        }
    }

    pub fn process_file<P>(
        &mut self, 
        path: P, 
        sender: SyncSender<InternalTrade> 
    ) -> Result<String, IngestError> 
    where
        P: AsRef<Path>,
    {
        let file = File::open(path)?;
        let reader = BufReader::with_capacity(64 * 1024, file);

        let mut csv_rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        let mut raw_record = ByteRecord::new();

        while csv_rdr.read_byte_record(&mut raw_record)? {
            let record: RawTradeRecord = raw_record.deserialize(None)?;
            
            self.update_hash(&record);

            // CONVERT IMMEDIATELY
            // We must own the data to send it across threads.
            // The 'RawTradeRecord' cannot leave this loop, so we convert to 'InternalTrade' here.
            let trade: InternalTrade = record.into();

            // SEND TO CHANNEL
            // If the buffer is full, this waits (Backpressure).
            // We unwrap because if the receiver hangs up, we should crash.
            sender.send(trade).expect("Receiver hung up");

            self.records_processed += 1;
        }

        Ok(self.finalize_hash())
    }

    fn update_hash(&mut self, record: &RawTradeRecord) {
        self.replay_hasher.update(record.slot.to_le_bytes());
        self.replay_hasher.update(record.tx_signature.as_bytes());
    }

    // Changed to &mut self to fix the "cannot move out of self" error
    pub fn finalize_hash(&mut self) -> String {
        // We clone the hasher so we don't destroy the state (just in case)
        let result = self.replay_hasher.clone().finalize();
        hex::encode(result)
    }
}