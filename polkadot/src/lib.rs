// Polkavex Substrate Pallet - Day 0 Simple Version
// This will be expanded in Day 2 with full Substrate/XCM integration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Basic escrow structure for Day 0
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Escrow {
    pub secret_hash: [u8; 32],
    pub maker: String,
    pub resolver: String,
    pub amount: u128,
    pub timelock: u64,
    pub completed: bool,
    pub cancelled: bool,
}

/// Simple in-memory storage for Day 0 (will use proper Substrate storage in Day 2)
pub struct PolkavexPallet {
    escrows: HashMap<[u8; 32], Escrow>,
}

impl PolkavexPallet {
    pub fn new() -> Self {
        Self {
            escrows: HashMap::new(),
        }
    }

    /// Create new escrow
    pub fn create_escrow(
        &mut self,
        secret_hash: [u8; 32],
        maker: String,
        resolver: String,
        amount: u128,
        timelock: u64,
    ) -> Result<(), String> {
        if self.escrows.contains_key(&secret_hash) {
            return Err("Escrow already exists".to_string());
        }

        let escrow = Escrow {
            secret_hash,
            maker,
            resolver,
            amount,
            timelock,
            completed: false,
            cancelled: false,
        };

        self.escrows.insert(secret_hash, escrow);
        println!("✅ Escrow created with hash: {:?}", secret_hash);
        Ok(())
    }

    /// Resolve escrow (basic version)
    pub fn resolve_escrow(&mut self, secret: [u8; 32]) -> Result<(), String> {
        // Simple hash calculation for Day 0
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        secret.hash(&mut hasher);
        let hash_result = hasher.finish();
        
        // Convert to [u8; 32] for lookup (simplified)
        let secret_hash = [0u8; 32]; // Placeholder for Day 0
        
        match self.escrows.get_mut(&secret_hash) {
            Some(escrow) => {
                escrow.completed = true;
                println!("✅ Escrow resolved");
                Ok(())
            }
            None => Err("Escrow not found".to_string()),
        }
    }

    /// Get escrow by hash
    pub fn get_escrow(&self, secret_hash: &[u8; 32]) -> Option<&Escrow> {
        self.escrows.get(secret_hash)
    }

    /// Get all escrows (for testing)
    pub fn get_all_escrows(&self) -> &HashMap<[u8; 32], Escrow> {
        &self.escrows
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_escrow() {
        let mut pallet = PolkavexPallet::new();
        let secret_hash = [1u8; 32];
        
        let result = pallet.create_escrow(
            secret_hash,
            "maker".to_string(),
            "resolver".to_string(),
            1000,
            12345,
        );
        
        assert!(result.is_ok());
        assert!(pallet.get_escrow(&secret_hash).is_some());
    }
}
