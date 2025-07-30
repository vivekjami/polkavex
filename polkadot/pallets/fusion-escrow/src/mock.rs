//! Mock runtime for testing the fusion-escrow pallet

use crate as pallet_fusion_escrow;
use frame_support::{
    construct_runtime, parameter_types,
    traits::{ConstU32, ConstU64},
    weights::Weight,
    PalletId,
};
use frame_system as system;
use sp_core::{H256, ConstU128};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup}, BuildStorage, Perbill,
};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        Assets: pallet_assets,
        FusionEscrow: pallet_fusion_escrow,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
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
    type MaxConsumers = frame_support::traits::ConstU32<16>;
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
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type RuntimeHoldReason = ();
    type MaxHolds = ();
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
        frame_system::EnsureRoot<u64>,
    >;
    type ForceOrigin = frame_system::EnsureRoot<u64>;
    type AssetDeposit = AssetDeposit;
    type AssetAccountDeposit = ConstU128<1>;
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
    pub const MaxEscrowsPerAccount: u32 = 100;
    pub const MinTimelock: u64 = 10; // 10 blocks
    pub const MaxTimelock: u64 = 100800; // ~7 days assuming 6s blocks
    pub const FusionEscrowPalletId: PalletId = PalletId(*b"plkv/esc");
}

// Mock XCM executor that always succeeds
pub struct MockXcmExecutor;
impl frame_support::traits::ExecuteXcm<RuntimeCall> for MockXcmExecutor {
    type Prepared = ();
    
    fn prepare(
        _message: Xcm<RuntimeCall>,
    ) -> Result<Self::Prepared, xcm::latest::Error> {
        Ok(())
    }
    
    fn execute(
        _origin: impl Into<MultiLocation>,
        _message: Xcm<RuntimeCall>,
        _id: &mut [u8; 32],
        _weight_credit: Weight,
    ) -> Result<Weight, xcm::latest::Error> {
        Ok(Weight::zero())
    }
    
    fn prepare_and_execute(
        _origin: impl Into<MultiLocation>,
        _message: Xcm<RuntimeCall>,
        _id: &mut [u8; 32],
        _weight_limit: Weight,
        _weight_credit: Weight,
    ) -> Result<Weight, xcm::latest::Error> {
        Ok(Weight::zero())
    }
}

// Mock time provider
pub struct MockTimeProvider;
impl frame_support::traits::UnixTime for MockTimeProvider {
    fn now() -> core::time::Duration {
        core::time::Duration::from_secs(0)
    }
}

impl pallet_fusion_escrow::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type Assets = Assets;
    type XcmExecutor = MockXcmExecutor;
    type WeightInfo = ();
    type MaxEscrowsPerAccount = MaxEscrowsPerAccount;
    type MinTimelock = MinTimelock;
    type MaxTimelock = MaxTimelock;
    type TimeProvider = MockTimeProvider;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = system::GenesisConfig::<Test>::default().build_storage().unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 10000000),
            (2, 10000000),
            (3, 10000000),
            (4, 10000000),
        ],
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    pallet_assets::GenesisConfig::<Test> {
        assets: vec![
            (0, 1, true, 1), // Asset 0, owner 1, is_sufficient true, min_balance 1
        ],
        metadata: vec![
            (0, b"Test Token".to_vec(), b"TEST".to_vec(), 12),
        ],
        accounts: vec![
            (0, 1, 1000000), // Asset 0, account 1, balance 1000000
            (0, 2, 1000000), // Asset 0, account 2, balance 1000000
        ],
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    storage.into()
}
