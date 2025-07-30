//! Weight definitions for pallet-fusion-escrow

use frame_support::weights::{Weight, constants::RocksDbWeight};

/// Weight functions for `pallet_fusion_escrow`.
pub trait WeightInfo {
    fn create_escrow() -> Weight;
    fn fund_escrow() -> Weight;
    fn complete_escrow() -> Weight;
    fn cancel_escrow() -> Weight;
    fn cancel_before_funding() -> Weight;
    fn toggle_pause() -> Weight;
}

/// Weights for pallet_fusion_escrow using the Substrate reference hardware.
impl WeightInfo for () {
    /// Storage: FusionEscrow NextEscrowId (r:1 w:1)
    /// Storage: FusionEscrow Escrows (r:0 w:1)
    /// Storage: FusionEscrow EscrowsBySecret (r:1 w:1)
    /// Storage: FusionEscrow EscrowsByMaker (r:1 w:1)
    /// Storage: FusionEscrow EscrowsByTaker (r:1 w:1)
    fn create_escrow() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(5))
    }

    /// Storage: FusionEscrow Escrows (r:1 w:1)
    /// Storage: Assets Account (r:2 w:2)
    /// Storage: System Account (r:1 w:1)
    fn fund_escrow() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(4))
    }

    /// Storage: FusionEscrow Escrows (r:1 w:1)
    /// Storage: Assets Account (r:2 w:2)
    /// Storage: System Account (r:1 w:1)
    fn complete_escrow() -> Weight {
        Weight::from_parts(45_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(4))
    }

    /// Storage: FusionEscrow Escrows (r:1 w:1)
    /// Storage: Assets Account (r:2 w:2)
    /// Storage: System Account (r:1 w:1)
    fn cancel_escrow() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(4))
    }

    /// Storage: FusionEscrow Escrows (r:1 w:1)
    /// Storage: FusionEscrow EscrowsBySecret (r:0 w:1)
    fn cancel_before_funding() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    /// Storage: FusionEscrow IsPaused (r:0 w:1)
    fn toggle_pause() -> Weight {
        Weight::from_parts(20_000_000, 0)
            .saturating_add(RocksDbWeight::get().writes(1))
    }
}
