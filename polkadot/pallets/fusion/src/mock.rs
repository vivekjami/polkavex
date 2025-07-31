//! Mock runtime for testing the Fusion pallet

use crate as pallet_fusion;
use frame_support::{
    construct_runtime, parameter_types,
    traits::{ConstU16, ConstU32, ConstU64, Everything},
    weights::Weight,
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
    pub struct Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>,
    {
        System: frame_system,
        Balances: pallet_balances,
        Assets: pallet_assets,
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
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = sp_runtime::generic::Header<u64, BlakeTwo256>;
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
}

impl pallet_balances::Config for Test {
    type MaxLocks = MaxLocks;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = u128;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
}

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
    type CreateOrigin = frame_support::traits::AsEnsureOriginWithArg<
        frame_system::EnsureSignedBy<frame_support::traits::IsInVec<AssetsAdmin>, u64>
    >;
    type ForceOrigin = frame_system::EnsureRoot<u64>;
    type AssetDeposit = AssetDeposit;
    type AssetAccountDeposit = ConstU32<1>;
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
    pub const AssetsAdmin: u64 = 1;
}

impl pallet_fusion::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Assets = Assets;
    type XcmTeleportFilter = ();
    type WeightInfo = ();
    type MaxEscrowsPerAccount = ConstU32<100>;
    type MinTimelockBlocks = ConstU32<10>;
    type MaxTimelockBlocks = ConstU32<518400>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
    
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
    ext.execute_with(|| System::set_block_number(1));
    ext
}
