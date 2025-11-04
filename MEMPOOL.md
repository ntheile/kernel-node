# Bitcoin Mempool Interface

This document describes the Bitcoin mempool interface added to the kernel-node project.

## Overview

The mempool (memory pool) is a data structure that stores unconfirmed transactions that have been broadcast to the Bitcoin network but have not yet been included in a block. This interface provides a comprehensive API for managing such transactions.

## Location

The mempool interface is defined in `src/mempool.rs`.

## Key Components

### Core Types

#### `FeeRate`
Represents the fee rate for a transaction in satoshis per virtual byte (sat/vB).

```rust
pub struct FeeRate {
    pub sat_per_vbyte: u64,
}
```

Methods:
- `new(sat_per_vbyte: u64)` - Create a new fee rate
- `calculate_fee(&self, weight: Weight) -> u64` - Calculate fee for a given weight

#### `MempoolEntry`
Represents a transaction in the mempool along with its metadata.

```rust
pub struct MempoolEntry {
    pub transaction: Transaction,
    pub fee_rate: FeeRate,
    pub fee: u64,
    pub time_added: SystemTime,
    pub depends_on: Vec<Txid>,
    pub vsize: u64,
}
```

#### `MempoolConfig`
Configuration parameters for the mempool.

```rust
pub struct MempoolConfig {
    pub max_size_bytes: usize,
    pub min_fee_rate: FeeRate,
    pub max_transactions: usize,
    pub enable_rbf: bool,
    pub max_transaction_size: usize,
    pub transaction_expiry_seconds: u64,
}
```

Default configuration:
- Maximum size: 300 MB
- Minimum fee rate: 1 sat/vB
- Maximum transactions: 300,000
- RBF enabled: true
- Maximum transaction size: 100 KB
- Transaction expiry: 14 days

### Enums

#### `RejectionReason`
Reasons why a transaction might be rejected from the mempool:
- `AlreadyInMempool` - Transaction already exists in the mempool
- `AlreadyInBlockchain` - Transaction is already in a block
- `FeeTooLow` - Fee rate is too low
- `TransactionTooLarge` - Transaction exceeds size limits
- `MempoolFull` - Mempool has reached capacity
- `Conflict` - Transaction conflicts with an existing transaction (double spend)
- `Invalid(String)` - Transaction fails consensus rules
- `MissingInputs` - Parent transactions are missing
- `RbfRulesFailed` - Replace-by-fee rules not satisfied

#### `MempoolEvent`
Events that can occur in the mempool:
- `TransactionAdded` - A transaction was added
- `TransactionRemoved` - A transaction was removed
- `TransactionReplaced` - A transaction was replaced (RBF)
- `SizeChanged` - Mempool size changed significantly

#### `RemovalReason`
Reasons for removing a transaction:
- `Mined` - Transaction was included in a block
- `Expired` - Transaction expired
- `Evicted` - Transaction was evicted due to low fee
- `Replaced` - Transaction was replaced by a higher fee transaction
- `Invalid` - Transaction became invalid
- `Manual` - Manually removed

### Traits

#### `Mempool`
The core mempool interface trait that any mempool implementation must satisfy.

**Key Methods:**

```rust
pub trait Mempool {
    // Add a transaction to the mempool
    fn add_transaction(&mut self, transaction: Transaction) -> MempoolResult<Txid>;
    
    // Remove a transaction from the mempool
    fn remove_transaction(&mut self, txid: &Txid, reason: RemovalReason) -> MempoolResult<Transaction>;
    
    // Check if a transaction exists
    fn contains(&self, txid: &Txid) -> bool;
    
    // Get a transaction
    fn get_transaction(&self, txid: &Txid) -> Option<&MempoolEntry>;
    
    // Get all transactions
    fn get_all_transactions(&self) -> HashMap<Txid, MempoolEntry>;
    
    // Get transactions by fee rate
    fn get_transactions_by_fee_rate(&self, min_fee_rate: FeeRate) -> Vec<MempoolEntry>;
    
    // Get mempool statistics
    fn get_stats(&self) -> MempoolStats;
    
    // Clear all transactions
    fn clear(&mut self);
    
    // Update mempool after a block
    fn update_for_block(&mut self, block_txids: &[Txid]);
    
    // Trim mempool to size limits
    fn trim(&mut self);
    
    // Remove expired transactions
    fn remove_expired(&mut self);
}
```

#### `MempoolExtended`
Extended interface with advanced features.

**Additional Methods:**

```rust
pub trait MempoolExtended: Mempool {
    // Query mempool with filters
    fn query(&self, query: MempoolQuery) -> Vec<MempoolEntry>;
    
    // Get descendant transactions
    fn get_descendants(&self, txid: &Txid) -> Vec<Txid>;
    
    // Get ancestor transactions
    fn get_ancestors(&self, txid: &Txid) -> Vec<Txid>;
    
    // Calculate package fee rate
    fn calculate_package_fee_rate(&self, txids: &[Txid]) -> MempoolResult<FeeRate>;
    
    // Register event listener
    fn add_event_listener(&mut self, listener: Box<dyn MempoolEventListener>);
}
```

#### `MempoolEventListener`
Trait for receiving mempool events.

```rust
pub trait MempoolEventListener {
    fn on_event(&mut self, event: MempoolEvent);
}
```

### Supporting Types

#### `MempoolStats`
Statistics about the current state of the mempool:
- `transaction_count` - Total number of transactions
- `total_bytes` - Total size in bytes
- `total_fees` - Total fees in satoshis
- `min_fee_rate` - Minimum fee rate
- `max_fee_rate` - Maximum fee rate
- `median_fee_rate` - Median fee rate
- `orphan_count` - Number of orphan transactions

#### `MempoolQuery`
Query parameters for filtering transactions:
- `min_fee_rate` - Minimum fee rate filter
- `max_fee_rate` - Maximum fee rate filter
- `min_size` - Minimum transaction size
- `max_size` - Maximum transaction size
- `limit` - Maximum number of results
- `sort_by_fee_rate` - Sort results by fee rate

#### `MempoolError`
Error type for mempool operations:
- `Rejected(RejectionReason)` - Transaction was rejected
- `NotFound(Txid)` - Transaction not found
- `Internal(String)` - Internal error

## Usage Example

```rust
use kernel_node::mempool::{Mempool, MempoolConfig, FeeRate};

// Create a mempool with custom configuration
let config = MempoolConfig {
    max_size_bytes: 100 * 1024 * 1024, // 100 MB
    min_fee_rate: FeeRate::new(5),     // 5 sat/vB minimum
    ..Default::default()
};

// In an implementation:
// let mut mempool = MyMempoolImplementation::new(config);
// 
// // Add a transaction
// match mempool.add_transaction(tx) {
//     Ok(txid) => println!("Added transaction: {}", txid),
//     Err(e) => println!("Rejected: {}", e),
// }
//
// // Get statistics
// let stats = mempool.get_stats();
// println!("Mempool has {} transactions", stats.transaction_count);
//
// // Update after a block
// mempool.update_for_block(&block_txids);
```

## Design Principles

1. **Interface-Only**: This module provides only trait definitions and data types. Actual implementations are left to users of this library.

2. **Flexibility**: The traits allow for multiple implementation strategies (e.g., in-memory, persistent, etc.).

3. **Observability**: Events and statistics provide visibility into mempool operations.

4. **Bitcoin Core Compatibility**: Design follows patterns similar to Bitcoin Core's mempool for familiarity.

5. **Type Safety**: Strong typing prevents common errors and makes the API self-documenting.

## Implementation Notes

To implement a mempool using this interface:

1. Create a struct to hold your mempool state
2. Implement the `Mempool` trait for basic functionality
3. Optionally implement `MempoolExtended` for advanced features
4. Handle transaction validation, fee calculation, and eviction policies

Example skeleton:

```rust
struct MyMempool {
    entries: HashMap<Txid, MempoolEntry>,
    config: MempoolConfig,
}

impl Mempool for MyMempool {
    fn add_transaction(&mut self, transaction: Transaction) -> MempoolResult<Txid> {
        // Implementation here
        todo!()
    }
    
    // ... other methods
}
```

## Testing

The module includes basic unit tests for core types like `FeeRate`. Implementers should add comprehensive tests for their specific implementations.

## Future Enhancements

Potential areas for future expansion:
- Package relay support (CPFP - Child Pays For Parent)
- Cluster mempool algorithms
- Better eviction policies
- Mempool persistence
- Network synchronization interfaces
- Fee estimation integration

## References

- [Bitcoin Core Mempool](https://github.com/bitcoin/bitcoin/blob/master/src/txmempool.h)
- [BIP 125: Replace-by-Fee](https://github.com/bitcoin/bips/blob/master/bip-0125.mediawiki)
- [Bitcoin Transaction Weight](https://en.bitcoin.it/wiki/Weight_units)
