//! Mock runtime for testing the Fusion pallet

use crate as pallet_fusion;
use frame_support::{
    construct_runtime, parameter_types,
    traits::{ConstU32, Everything},
    PalletId,
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
    pub struct Test {
        System: frame_system,
        Balances: pallet_balances,
        Assets: pallet_assets,
        Randomness: pallet_insecure_randomness_collective_flip,
        Timestamp: pallet_timestamp,
        Fusion: pallet_fusion,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
    type BaseCallFilter = Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Block = Block;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 500;
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    type Balance = u128;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
    type RuntimeHoldReason = ();
    type FreezeIdentifier = ();
    type MaxHolds = ConstU32<1>;
    type MaxFreezes = ConstU32<1>;
}

parameter_types! {
    pub const MinimumPeriod: u64 = 1;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

impl pallet_insecure_randomness_collective_flip::Config for Test {}

parameter_types! {
    pub const AssetDeposit: u128 = 100;
    pub const ApprovalDeposit: u128 = 1;
    pub const StringLimit: u32 = 50;
    pub const MetadataDepositBase: u128 = 10;
    pub const MetadataDepositPerByte: u128 = 1;
}

impl pallet_assets::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = u128;
    type AssetId = u32;
    type AssetIdParameter = codec::Compact<u32>;
    type Currency = Balances;
    type CreateOrigin = frame_support::traits::AsEnsureOriginWithArg<frame_system::EnsureSigned<u64>>;
    type ForceOrigin = frame_system::EnsureRoot<u64>;
    type AssetDeposit = AssetDeposit;
    type AssetAccountDeposit = ConstU32<100>;
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type ApprovalDeposit = ApprovalDeposit;
    type StringLimit = StringLimit;
    type Freezer = ();
    type Extra = ();
    type CallbackHandle = ();
    type WeightInfo = pallet_assets::weights::SubstrateWeight<Test>;
    type RemoveItemsLimit = ConstU32<1000>;
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = ();
}

parameter_types! {
    pub const FusionPalletId: PalletId = PalletId(*b"py/fusio");
    pub const MaxEscrowsPerAccount: u32 = 100;
    pub const MinTimelockDuration: u64 = 10; // 10 blocks minimum
    pub const MaxTimelockDuration: u64 = 1_000_000; // ~7 days at 6 second blocks
    pub const EscrowFee: u128 = 1_000_000_000_000; // 1 DOT fee
}

impl pallet_fusion::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type EscrowId = u64;
    type AssetId = u32;
    type Balance = u128;
    type Assets = Assets;
    type Currency = Balances;
    type TimeProvider = Timestamp;
    type Randomness = Randomness;
    type PalletId = FusionPalletId;
    type MaxEscrowsPerAccount = MaxEscrowsPerAccount;
    type MinTimelockDuration = MinTimelockDuration;
    type MaxTimelockDuration = MaxTimelockDuration;
    type EscrowFee = EscrowFee;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
    
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 1_000_000_000_000_000), // 1M DOT
            (2, 1_000_000_000_000_000), // 1M DOT
            (3, 1_000_000_000_000_000), // 1M DOT
        ],
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(storage);
    ext.execute_with(|| {
        System::set_block_number(1);
        Timestamp::set_timestamp(12345);
    });
    ext
}

// Test accounts
pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;
pub const DAVE: u64 = 4;

// Helper functions for tests
pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        let block_number = System::block_number() + 1;
        System::set_block_number(block_number);
        Timestamp::set_timestamp(block_number * 6000); // 6 seconds per block
        System::on_finalize(System::block_number());
        System::on_initialize(block_number);
    }
}

pub fn last_event() -> RuntimeEvent {
    System::events().pop().expect("Event expected").event
}
