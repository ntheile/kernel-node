//! Bitcoin Mempool Interface
//!
//! This module provides interfaces and types for managing a Bitcoin mempool.
//! A mempool (memory pool) stores unconfirmed transactions that have been
//! broadcast to the network but not yet included in a block.

use bitcoin::{Transaction, Txid, Weight};
use std::collections::HashMap;
use std::time::SystemTime;

/// Represents the fee rate for a transaction in satoshis per virtual byte (sat/vB)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FeeRate {
    /// Fee rate in satoshis per virtual byte
    pub sat_per_vbyte: u64,
}

impl FeeRate {
    /// Create a new fee rate
    pub fn new(sat_per_vbyte: u64) -> Self {
        Self { sat_per_vbyte }
    }

    /// Calculate fee for a given weight
    pub fn calculate_fee(&self, weight: Weight) -> u64 {
        // Convert weight to virtual bytes (weight / 4)
        let vbytes = (weight.to_wu() + 3) / 4;
        self.sat_per_vbyte * vbytes
    }
}

/// Entry in the mempool representing a transaction and its metadata
#[derive(Debug, Clone)]
pub struct MempoolEntry {
    /// The transaction
    pub transaction: Transaction,
    
    /// Fee rate of this transaction
    pub fee_rate: FeeRate,
    
    /// Total fee paid by this transaction in satoshis
    pub fee: u64,
    
    /// Time when the transaction was added to the mempool
    pub time_added: SystemTime,
    
    /// Transaction dependencies (parent transactions in the mempool)
    pub depends_on: Vec<Txid>,
    
    /// Transaction size in virtual bytes
    pub vsize: u64,
}

/// Configuration parameters for the mempool
#[derive(Debug, Clone)]
pub struct MempoolConfig {
    /// Maximum size of the mempool in bytes
    pub max_size_bytes: usize,
    
    /// Minimum fee rate to accept transactions (sat/vB)
    pub min_fee_rate: FeeRate,
    
    /// Maximum number of transactions in the mempool
    pub max_transactions: usize,
    
    /// Whether to replace-by-fee (RBF) is enabled
    pub enable_rbf: bool,
    
    /// Maximum transaction size in bytes
    pub max_transaction_size: usize,
    
    /// Expiry time for transactions in seconds
    pub transaction_expiry_seconds: u64,
}

impl Default for MempoolConfig {
    fn default() -> Self {
        Self {
            max_size_bytes: 300 * 1024 * 1024, // 300 MB
            min_fee_rate: FeeRate::new(1),      // 1 sat/vB
            max_transactions: 300_000,
            enable_rbf: true,
            max_transaction_size: 100_000,      // 100 KB
            transaction_expiry_seconds: 14 * 24 * 60 * 60, // 14 days
        }
    }
}

/// Reasons why a transaction might be rejected from the mempool
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RejectionReason {
    /// Transaction already exists in the mempool
    AlreadyInMempool,
    
    /// Transaction is already in a block
    AlreadyInBlockchain,
    
    /// Fee rate is too low
    FeeTooLow,
    
    /// Transaction is too large
    TransactionTooLarge,
    
    /// Mempool is full
    MempoolFull,
    
    /// Transaction conflicts with an existing transaction (double spend)
    Conflict,
    
    /// Invalid transaction (fails consensus rules)
    Invalid(String),
    
    /// Missing parent transactions
    MissingInputs,
    
    /// Replace-by-fee rules not satisfied
    RbfRulesFailed,
}

/// Events that can occur in the mempool
#[derive(Debug, Clone)]
pub enum MempoolEvent {
    /// A transaction was added to the mempool
    TransactionAdded {
        txid: Txid,
        fee_rate: FeeRate,
    },
    
    /// A transaction was removed from the mempool
    TransactionRemoved {
        txid: Txid,
        reason: RemovalReason,
    },
    
    /// A transaction was replaced by another (RBF)
    TransactionReplaced {
        old_txid: Txid,
        new_txid: Txid,
    },
    
    /// Mempool size changed significantly
    SizeChanged {
        transaction_count: usize,
        total_bytes: usize,
    },
}

/// Reasons for removing a transaction from the mempool
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemovalReason {
    /// Transaction was included in a block
    Mined,
    
    /// Transaction expired
    Expired,
    
    /// Transaction was evicted due to low fee
    Evicted,
    
    /// Transaction was replaced by a higher fee transaction
    Replaced,
    
    /// Transaction became invalid (e.g., parent was removed)
    Invalid,
    
    /// Manually removed
    Manual,
}

/// Statistics about the current state of the mempool
#[derive(Debug, Clone)]
pub struct MempoolStats {
    /// Total number of transactions in the mempool
    pub transaction_count: usize,
    
    /// Total size of all transactions in bytes
    pub total_bytes: usize,
    
    /// Total fees of all transactions in satoshis
    pub total_fees: u64,
    
    /// Minimum fee rate among all transactions
    pub min_fee_rate: FeeRate,
    
    /// Maximum fee rate among all transactions
    pub max_fee_rate: FeeRate,
    
    /// Median fee rate
    pub median_fee_rate: FeeRate,
    
    /// Number of orphan transactions (missing parents)
    pub orphan_count: usize,
}

/// Result type for mempool operations
pub type MempoolResult<T> = Result<T, MempoolError>;

/// Errors that can occur during mempool operations
#[derive(Debug, Clone)]
pub enum MempoolError {
    /// Transaction was rejected
    Rejected(RejectionReason),
    
    /// Transaction not found in mempool
    NotFound(Txid),
    
    /// Internal error
    Internal(String),
}

impl std::fmt::Display for MempoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MempoolError::Rejected(reason) => write!(f, "Transaction rejected: {:?}", reason),
            MempoolError::NotFound(txid) => write!(f, "Transaction not found: {:?}", txid),
            MempoolError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for MempoolError {}

/// Core trait defining the interface for a Bitcoin mempool
pub trait Mempool {
    /// Add a transaction to the mempool
    ///
    /// # Arguments
    /// * `transaction` - The transaction to add
    ///
    /// # Returns
    /// * `Ok(Txid)` - The transaction ID if successfully added
    /// * `Err(MempoolError)` - Error if the transaction was rejected
    fn add_transaction(&mut self, transaction: Transaction) -> MempoolResult<Txid>;
    
    /// Remove a transaction from the mempool
    ///
    /// # Arguments
    /// * `txid` - The transaction ID to remove
    /// * `reason` - The reason for removal
    ///
    /// # Returns
    /// * `Ok(Transaction)` - The removed transaction
    /// * `Err(MempoolError)` - Error if transaction not found
    fn remove_transaction(&mut self, txid: &Txid, reason: RemovalReason) -> MempoolResult<Transaction>;
    
    /// Check if a transaction exists in the mempool
    ///
    /// # Arguments
    /// * `txid` - The transaction ID to check
    ///
    /// # Returns
    /// * `true` if the transaction exists, `false` otherwise
    fn contains(&self, txid: &Txid) -> bool;
    
    /// Get a transaction from the mempool
    ///
    /// # Arguments
    /// * `txid` - The transaction ID to retrieve
    ///
    /// # Returns
    /// * `Some(MempoolEntry)` - The mempool entry if found
    /// * `None` - If transaction not found
    fn get_transaction(&self, txid: &Txid) -> Option<&MempoolEntry>;
    
    /// Get all transactions in the mempool
    ///
    /// # Returns
    /// * Map of transaction IDs to mempool entries
    ///
    /// # Note
    /// This returns owned data which may require cloning for large mempools.
    /// Implementations may optimize this by using reference-counted pointers internally.
    fn get_all_transactions(&self) -> HashMap<Txid, MempoolEntry>;
    
    /// Get transactions with fee rate above a threshold
    ///
    /// # Arguments
    /// * `min_fee_rate` - Minimum fee rate threshold
    ///
    /// # Returns
    /// * Vector of mempool entries matching the criteria
    ///
    /// # Note
    /// Returns owned data. For large result sets, consider using iteration or pagination.
    fn get_transactions_by_fee_rate(&self, min_fee_rate: FeeRate) -> Vec<MempoolEntry>;
    
    /// Get current mempool statistics
    ///
    /// # Returns
    /// * Current mempool statistics
    fn get_stats(&self) -> MempoolStats;
    
    /// Clear all transactions from the mempool
    fn clear(&mut self);
    
    /// Update mempool after a block is added to the chain
    ///
    /// # Arguments
    /// * `block_txids` - Transaction IDs that were included in the block
    fn update_for_block(&mut self, block_txids: &[Txid]);
    
    /// Trim the mempool to fit within size limits
    ///
    /// This should evict lowest fee transactions as needed
    fn trim(&mut self);
    
    /// Remove expired transactions
    fn remove_expired(&mut self);
}

/// Trait for mempool event listeners
pub trait MempoolEventListener {
    /// Called when a mempool event occurs
    ///
    /// # Arguments
    /// * `event` - The event that occurred
    fn on_event(&mut self, event: MempoolEvent);
}

/// Query options for retrieving transactions from the mempool
#[derive(Debug, Clone, Default)]
pub struct MempoolQuery {
    /// Minimum fee rate
    pub min_fee_rate: Option<FeeRate>,
    
    /// Maximum fee rate
    pub max_fee_rate: Option<FeeRate>,
    
    /// Minimum transaction size
    pub min_size: Option<u64>,
    
    /// Maximum transaction size
    pub max_size: Option<u64>,
    
    /// Maximum number of results to return
    pub limit: Option<usize>,
    
    /// Sort by fee rate (descending if true)
    pub sort_by_fee_rate: bool,
}

/// Extended mempool trait with advanced features
pub trait MempoolExtended: Mempool {
    /// Query the mempool with advanced filters
    ///
    /// # Arguments
    /// * `query` - Query parameters
    ///
    /// # Returns
    /// * Vector of matching mempool entries
    ///
    /// # Note
    /// Returns owned data. Use the `limit` field in MempoolQuery to control result size.
    fn query(&self, query: MempoolQuery) -> Vec<MempoolEntry>;
    
    /// Get descendant transactions (transactions that depend on this one)
    ///
    /// # Arguments
    /// * `txid` - The transaction ID
    ///
    /// # Returns
    /// * Vector of descendant transaction IDs
    fn get_descendants(&self, txid: &Txid) -> Vec<Txid>;
    
    /// Get ancestor transactions (transactions this one depends on)
    ///
    /// # Arguments
    /// * `txid` - The transaction ID
    ///
    /// # Returns
    /// * Vector of ancestor transaction IDs
    fn get_ancestors(&self, txid: &Txid) -> Vec<Txid>;
    
    /// Calculate the fee rate for a transaction package
    ///
    /// # Arguments
    /// * `txids` - Transaction IDs to include in the package
    ///
    /// # Returns
    /// * Package fee rate or error
    fn calculate_package_fee_rate(&self, txids: &[Txid]) -> MempoolResult<FeeRate>;
    
    /// Register an event listener
    ///
    /// # Arguments
    /// * `listener` - Event listener to register
    fn add_event_listener(&mut self, listener: Box<dyn MempoolEventListener>);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fee_rate_calculation() {
        let fee_rate = FeeRate::new(10);
        let weight = Weight::from_wu(400); // 100 vbytes
        let fee = fee_rate.calculate_fee(weight);
        assert_eq!(fee, 1000); // 10 sat/vB * 100 vB = 1000 sat
    }

    #[test]
    fn test_fee_rate_ordering() {
        let low = FeeRate::new(1);
        let high = FeeRate::new(10);
        assert!(low < high);
        assert!(high > low);
    }

    #[test]
    fn test_default_config() {
        let config = MempoolConfig::default();
        assert_eq!(config.max_size_bytes, 300 * 1024 * 1024);
        assert_eq!(config.min_fee_rate.sat_per_vbyte, 1);
        assert!(config.enable_rbf);
    }
}
