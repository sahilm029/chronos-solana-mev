use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use sha2::{Sha256, Digest};
use csv::ByteRecord; // We need this for manual buffer management

// Import the struct from our types module
use crate::types::RawTradeRecord;

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

    pub fn process_file<P, F>(
        &mut self, 
        path: P, 
        mut on_record: F
    ) -> Result<String, IngestError> 
    where
        P: AsRef<Path>,
        F: FnMut(RawTradeRecord) -> (), 
    {
        let file = File::open(path)?;
        let reader = BufReader::with_capacity(64 * 1024, file);

        let mut csv_rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        // MANUAL BUFFER MANAGEMENT (The Fix)
        // We create a reusable buffer. This is much faster than allocating a new one every row.
        let mut raw_record = ByteRecord::new();

        // We loop by reading into this existing buffer
        while csv_rdr.read_byte_record(&mut raw_record)? {
            
            // We deserialize strictly from the 'raw_record' buffer
            // This is allowed because 'record' dies at the end of the loop,
            // just before 'raw_record' is overwritten.
            let record: RawTradeRecord = raw_record.deserialize(None)?;

            self.update_hash(&record);
            on_record(record);

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