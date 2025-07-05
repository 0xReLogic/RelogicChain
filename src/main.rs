
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use chrono::prelude::*;
use std::time::{Duration, Instant};
use rayon::prelude::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::atomic::{AtomicBool, Ordering, AtomicU64};
use std::sync::Arc;

// --- Konstanta & Konfigurasi ---
const INITIAL_DIFFICULTY: u32 = 15; // Kesulitan awal yang lebih menantang
const BLOCK_TIME_SECONDS: u64 = 10; // Target waktu per blok
const DIFFICULTY_ADJUSTMENT_INTERVAL: u64 = 10; // Penyesuaian setiap 10 blok
const INITIAL_REWARD: u64 = 50; // Hadiah awal
const HALVING_INTERVAL: u64 = 20; // Halving setiap 20 blok (untuk demonstrasi)

// --- Error Handling ---
#[derive(Debug)]
pub enum MiningError {
    Interrupted,
    NoValidNonceFound,
}

// --- Transaction ---
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Transaction {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub timestamp: u64,
    pub signature: String,
}

impl Transaction {
    /// Membuat transaksi baru.
    pub fn new(from: String, to: String, amount: u64, signature: String) -> Self {
        let timestamp = Utc::now().timestamp_millis() as u64;
        let mut transaction = Self {
            id: String::new(),
            from,
            to,
            amount,
            timestamp,
            signature,
        };
        transaction.id = transaction.calculate_hash();
        transaction
    }

    /// Menghitung hash dari transaksi.
    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        let record = format!("{}{}{}{}{}", self.from, self.to, self.amount, self.timestamp, self.signature);
        hasher.update(record.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Membuat transaksi coinbase untuk hadiah mining.
    pub fn coinbase(to: String, amount: u64) -> Self {
        Transaction::new("coinbase".to_string(), to, amount, "".to_string())
    }
}

// --- Block ---
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub previous_hash: String,
    pub hash: String,
    pub merkle_root: String,
    pub nonce: u64,
    pub difficulty: u32,
    pub transactions: Vec<Transaction>,
}

impl Block {
    /// Membuat instance blok baru (tanpa hash).
    fn new(index: u64, previous_hash: String, difficulty: u32, transactions: Vec<Transaction>) -> Self {
        let timestamp = Utc::now().timestamp_millis() as u64;
        let merkle_root = MerkleTree::new(&transactions).build_tree();
        
        Self {
            index,
            timestamp,
            previous_hash,
            hash: String::new(),
            merkle_root,
            nonce: 0,
            difficulty,
            transactions,
        }
    }

    /// Menghitung hash untuk blok.
    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        let record = format!("{}{}{}{}{}{}", self.index, self.timestamp, self.previous_hash, self.merkle_root, self.nonce, self.difficulty);
        hasher.update(record.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

// --- Merkle Tree ---
#[derive(Debug, Clone)]
pub struct MerkleTree {
    root: Option<String>,
    leaves: Vec<String>,
}

impl MerkleTree {
    /// Membuat Merkle Tree baru dari transaksi.
    pub fn new(transactions: &[Transaction]) -> Self {
        let leaves = transactions.iter().map(|tx| tx.id.clone()).collect();
        Self { root: None, leaves }
    }

    /// Membangun tree dan mengembalikan root hash.
    pub fn build_tree(&mut self) -> String {
        if self.leaves.is_empty() {
            return "0".repeat(64);
        }
        if self.leaves.len() == 1 {
            return self.leaves[0].clone();
        }

        let mut current_level = self.leaves.clone();
        while current_level.len() > 1 {
            if current_level.len() % 2 != 0 {
                current_level.push(current_level.last().unwrap().clone());
            }
            
            let mut next_level = Vec::new();
            for i in (0..current_level.len()).step_by(2) {
                let left = &current_level[i];
                let right = &current_level[i+1];
                let mut hasher = Sha256::new();
                hasher.update(left.as_bytes());
                hasher.update(right.as_bytes());
                next_level.push(format!("{:x}", hasher.finalize()));
            }
            current_level = next_level;
        }
        self.root = Some(current_level.remove(0));
        self.root.as_ref().unwrap().clone()
    }
}

// --- Blockchain ---
#[derive(Debug)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    miner_address: String,
    total_supply: u64,
}

impl Blockchain {
    /// Membuat blockchain baru.
    pub fn new(miner_address: String) -> Self {
        let mut chain = Self {
            blocks: Vec::new(),
            pending_transactions: Vec::new(),
            miner_address,
            total_supply: 0,
        };
        let mut genesis_block = chain.create_genesis_block();
        mine_block(&mut genesis_block, INITIAL_DIFFICULTY).expect("Gagal menambang blok genesis");
        chain.total_supply += chain.get_reward(0);
        chain.blocks.push(genesis_block);
        chain
    }

    /// Membuat blok genesis.
    fn create_genesis_block(&self) -> Block {
        let reward = self.get_reward(0);
        let coinbase_tx = Transaction::coinbase(self.miner_address.clone(), reward);
        Block::new(0, "0".repeat(64), INITIAL_DIFFICULTY, vec![coinbase_tx])
    }

    /// Menghitung hadiah mining berdasarkan tinggi blok.
    pub fn get_reward(&self, block_index: u64) -> u64 {
        let halvings = block_index / HALVING_INTERVAL;
        INITIAL_REWARD / (2u64.pow(halvings as u32))
    }

    /// Menyesuaikan kesulitan mining.
    pub fn adjust_difficulty(&self) -> u32 {
        if self.blocks.len() < DIFFICULTY_ADJUSTMENT_INTERVAL as usize {
            return self.blocks.last().unwrap().difficulty;
        }
        
        let last_adjustment_block = &self.blocks[self.blocks.len() - DIFFICULTY_ADJUSTMENT_INTERVAL as usize];
        let current_block = self.blocks.last().unwrap();
        
        let time_taken = current_block.timestamp - last_adjustment_block.timestamp;
        let expected_time = (DIFFICULTY_ADJUSTMENT_INTERVAL * BLOCK_TIME_SECONDS * 1000) as u64;

        let time_ratio = expected_time as f64 / time_taken as f64;
        let old_difficulty = current_block.difficulty as f64;

        // Batasi perubahan difficulty (misal, max 4x)
        let new_difficulty = if time_ratio > 4.0 {
            old_difficulty * 4.0
        } else if time_ratio < 0.25 {
            old_difficulty * 0.25
        } else {
            old_difficulty * time_ratio
        };
        
        // Batasi difficulty minimum
        (new_difficulty.round() as u32).max(1)
    }

    /// Menambang blok baru dan menambahkannya ke rantai.
    pub fn mine_and_add_block(&mut self) -> Result<(), MiningError> {
        let reward = self.get_reward(self.blocks.len() as u64);
        let mut transactions = self.pending_transactions.drain(..).collect::<Vec<_>>();
        transactions.insert(0, Transaction::coinbase(self.miner_address.clone(), reward));

        let difficulty = self.adjust_difficulty();
        let mut new_block = Block::new(
            self.blocks.len() as u64,
            self.blocks.last().unwrap().hash.clone(),
            difficulty,
            transactions
        );

        mine_block(&mut new_block, difficulty)?;
        
        println!("
Blok #{} berhasil ditambang!", new_block.index);
        println!("  Hash: {}", new_block.hash);
        println!("  Nonce: {}", new_block.nonce);
        println!("  Kesulitan: {}", new_block.difficulty);
        println!("  Hadiah: {}", reward);

        self.total_supply += reward;
        self.blocks.push(new_block);
        Ok(())
    }
}

/// Fungsi untuk menambang sebuah blok.
pub fn mine_block(block: &mut Block, difficulty: u32) -> Result<(), MiningError> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Gagal memasang handler Ctrl-C");

    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] {msg}").unwrap());
    
    let hashes_done = Arc::new(AtomicU64::new(0));
    let start_time = Instant::now();

    let target_prefix = "0".repeat(difficulty as usize);

    let found_nonce = (0..u64::MAX).into_par_iter().find_any(|&nonce| {
        if !running.load(Ordering::SeqCst) {
            return true;
        }
        
        let mut block_clone = block.clone();
        block_clone.nonce = nonce;
        let hash = block_clone.calculate_hash();
        
        let hashes = hashes_done.fetch_add(1, Ordering::SeqCst);
        if hashes % 1000 == 0 { // Update progress bar sesekali
            let elapsed_secs = start_time.elapsed().as_secs_f64();
            let hps = hashes as f64 / elapsed_secs.max(1.0);
            pb.set_message(format!("Mencari... ({} H/s)", hps as u64));
        }

        hash.starts_with(&target_prefix)
    });

    pb.finish_and_clear();

    if !running.load(Ordering::SeqCst) {
        return Err(MiningError::Interrupted);
    }

    match found_nonce {
        Some(nonce) => {
            block.nonce = nonce;
            block.hash = block.calculate_hash();
            Ok(())
        }
        None => Err(MiningError::NoValidNonceFound),
    }
}

// --- CLI ---
fn main() {
    println!("Membuat blockchain baru...");
    let mut blockchain = Blockchain::new("miner-utama".to_string());
    println!("Blok Genesis berhasil dibuat.");
    println!("Hash: {}", blockchain.blocks[0].hash);
    println!("Total Supply Awal: {}", blockchain.total_supply);
    println!("---");
    println!("Tekan Ctrl+C untuk menghentikan mining.");

    loop {
        println!("
Memulai penambangan untuk blok #{}...", blockchain.blocks.len());
        println!("Kesulitan saat ini: {}", blockchain.adjust_difficulty());
        println!("Total Supply: {}", blockchain.total_supply);
        
        // Tambahkan beberapa transaksi dummy
        blockchain.pending_transactions.push(Transaction::new("Alice".into(), "Bob".into(), 10, "sig".into()));
        blockchain.pending_transactions.push(Transaction::new("Charlie".into(), "David".into(), 5, "sig".into()));

        match blockchain.mine_and_add_block() {
            Ok(_) => {
                // Lanjutkan loop
            }
            Err(MiningError::Interrupted) => {
                println!("
Proses mining dihentikan oleh pengguna.");
                break;
            }
            Err(e) => {
                println!("
Terjadi error saat mining: {:?}", e);
                break;
            }
        }
    }
}

// --- Unit Tests ---
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_hash() {
        let tx = Transaction::new("a".into(), "b".into(), 10, "s".into());
        assert!(!tx.id.is_empty());
    }

    #[test]
    fn test_merkle_tree() {
        let txs = vec![
            Transaction::new("a".into(), "b".into(), 1, "s1".into()),
            Transaction::new("c".into(), "d".into(), 2, "s2".into()),
        ];
        let mut tree = MerkleTree::new(&txs);
        let root = tree.build_tree();
        assert!(!root.is_empty());
        assert_ne!(root, "0".repeat(64));
    }

    #[test]
    fn test_mining_and_valid_proof() {
        let mut block = Block::new(1, "prev_hash".into(), 5, vec![]);
        assert!(mine_block(&mut block, 5).is_ok());
        assert!(block.hash.starts_with(&"0".repeat(5)));
    }

    #[test]
    fn test_reward_halving() {
        let chain = Blockchain::new("test".into());
        assert_eq!(chain.get_reward(0), INITIAL_REWARD);
        assert_eq!(chain.get_reward(HALVING_INTERVAL - 1), INITIAL_REWARD);
        assert_eq!(chain.get_reward(HALVING_INTERVAL), INITIAL_REWARD / 2);
        assert_eq!(chain.get_reward(HALVING_INTERVAL * 2), INITIAL_REWARD / 4);
    }

    #[test]
    fn test_difficulty_adjustment_increase() {
        let mut chain = Blockchain::new("test".into());
        let initial_difficulty = chain.blocks[0].difficulty;
        
        // Simulasikan blok yang sangat cepat
        for i in 1..=DIFFICULTY_ADJUSTMENT_INTERVAL {
            let mut last_block = chain.blocks.last().unwrap().clone();
            last_block.index = i;
            // Kurangi 5 detik dari timestamp sebelumnya
            last_block.timestamp = last_block.timestamp - 5000; 
            chain.blocks.push(last_block);
        }
        
        let new_difficulty = chain.adjust_difficulty();
        assert!(new_difficulty > initial_difficulty, "Kesulitan seharusnya meningkat");
    }

    #[test]
    fn test_difficulty_adjustment_decrease() {
        let mut chain = Blockchain::new("test".into());
        let initial_difficulty = chain.blocks[0].difficulty;
        
        // Simulasikan blok yang sangat lambat
        for i in 1..=DIFFICULTY_ADJUSTMENT_INTERVAL {
            let mut last_block = chain.blocks.last().unwrap().clone();
            last_block.index = i;
            // Tambah 20 detik dari timestamp sebelumnya
            last_block.timestamp = last_block.timestamp + 20000; 
            chain.blocks.push(last_block);
        }
        
        let new_difficulty = chain.adjust_difficulty();
        assert!(new_difficulty < initial_difficulty, "Kesulitan seharusnya menurun");
    }
}
