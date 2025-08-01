#![cfg_attr(not(feature = "std"), no_std)]

//! # Polkavex Fusion Pallet
//!
//! A production-grade Substrate pallet implementing hashlock/timelock escrow logic
//! with multi-asset support and XCM integration for cross-chain DeFi operations.
//! 
//! This pallet provides compatibility with 1inch Fusion+ architecture, enabling
//! secure cross-chain atomic swaps with DOT, parachain tokens, and future NFT support.

use frame_support::{
    codec::{Decode, Encode},
    dispatch::DispatchResult,
    traits::{Get, Time, fungibles::Inspect, fungibles::Mutate, Randomness, tokens::Preservation, 
             Currency, ReservableCurrency, ExistenceRequirement},
    PalletId,
    pallet_prelude::*,
    storage::bounded_vec::BoundedVec,
};
use frame_system::pallet_prelude::*;
use sp_runtime::{
    traits::{AccountIdConversion, Saturating, Zero, CheckedAdd},
};
use sp_std::vec::Vec;
use scale_info::TypeInfo;
use sp_core::H256;

pub use pallet::*;

// Re-export weights module
pub mod weights;
pub use weights::WeightInfo;

// Import test utilities when building for tests
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// Maximum number of concurrent escrows per account
pub const MAX_ESCROWS_PER_ACCOUNT: u32 = 1000;

/// Maximum hashlock size (32 bytes for SHA256)
pub const MAX_HASHLOCK_SIZE: u32 = 32;

/// Maximum metadata size for escrow descriptions
pub const MAX_METADATA_SIZE: u32 = 1024;

/// Escrow state enumeration
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum EscrowState {
    /// Escrow created but not yet funded
    Created,
    /// Escrow funded and active
    Active,
    /// Escrow completed successfully
    Completed,
    /// Escrow cancelled or expired
    Cancelled,
    /// Emergency pause state
    Paused,
}

impl Default for EscrowState {
    fn default() -> Self {
        Self::Created
    }
}

/// Asset information for multi-asset support
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum AssetInfo<AssetId> {
    /// Native DOT token
    Native,
    /// Parachain fungible asset
    Asset(AssetId),
    /// Day 5: Stablecoin with enhanced metadata
    Stablecoin { asset_id: AssetId, decimals: u8, symbol: BoundedVec<u8, ConstU32<32>> },
    /// NFT support for Day 5 enhancements
    Nft { collection_id: AssetId, item_id: u32, metadata: BoundedVec<u8, ConstU32<256>> },
}

/// Asset type classification for routing optimization
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum AssetType {
    Native,
    Fungible,
    Stablecoin,
    Nft,
}

impl<AssetId> AssetInfo<AssetId> {
    /// Get the asset type for routing decisions
    pub fn asset_type(&self) -> AssetType {
        match self {
            AssetInfo::Native => AssetType::Native,
            AssetInfo::Asset(_) => AssetType::Fungible,
            AssetInfo::Stablecoin { .. } => AssetType::Stablecoin,
            AssetInfo::Nft { .. } => AssetType::Nft,
        }
    }
}

/// XCM routing information for cross-chain operations
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct XcmRoute {
    /// Destination parachain
    pub destination: sp_std::vec::Vec<u8>, // Simplified for now
    /// Additional routing data
    pub route_data: BoundedVec<u8, ConstU32<256>>,
}

/// Core escrow structure
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct Escrow<T: Config> {
    /// Unique escrow identifier
    pub id: T::EscrowId,
    /// Creator of the escrow
    pub creator: T::AccountId,
    /// Beneficiary who can claim the funds
    pub beneficiary: T::AccountId,
    /// Asset being escrowed
    pub asset: AssetInfo<T::AssetId>,
    /// Amount being escrowed
    pub amount: T::Balance,
    /// Hash lock for atomic swaps
    pub hashlock: BoundedVec<u8, ConstU32<MAX_HASHLOCK_SIZE>>,
    /// Time lock expiration
    pub timelock: BlockNumberFor<T>,
    /// Current state of the escrow
    pub state: EscrowState,
    /// Optional metadata
    pub metadata: BoundedVec<u8, ConstU32<MAX_METADATA_SIZE>>,
    /// XCM routing for cross-chain operations
    pub xcm_route: Option<XcmRoute>,
    /// Block when escrow was created
    pub created_at: BlockNumberFor<T>,
    /// Block when escrow was last updated
    pub updated_at: BlockNumberFor<T>,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// Configuration trait for the pallet
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Weight information for extrinsics
        type WeightInfo: WeightInfo;

        /// Type for escrow identifiers
        type EscrowId: Parameter + Copy + Default + MaxEncodedLen + From<u64> + CheckedAdd;

        /// Type for asset identifiers
        type AssetId: Parameter + Copy + Default + MaxEncodedLen;

        /// Type for balances
        type Balance: Parameter + Copy + Default + MaxEncodedLen + Zero + Saturating + CheckedAdd;

        /// Multi-asset support
        type Assets: Inspect<Self::AccountId, AssetId = Self::AssetId, Balance = Self::Balance>
            + Mutate<Self::AccountId>;

        /// Native currency operations using the traditional Currency trait
        type Currency: Currency<Self::AccountId, Balance = Self::Balance> + ReservableCurrency<Self::AccountId>;

        /// Time provider for timelock functionality  
        type TimeProvider: Time<Moment = BlockNumberFor<Self>>;

        /// Randomness source for generating secure escrow IDs
        type Randomness: Randomness<H256, BlockNumberFor<Self>>;

        /// Pallet ID for generating account addresses
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Maximum number of concurrent escrows per account
        #[pallet::constant]
        type MaxEscrowsPerAccount: Get<u32>;

        /// Minimum timelock duration in blocks
        #[pallet::constant]
        type MinTimelockDuration: Get<BlockNumberFor<Self>>;

        /// Maximum timelock duration in blocks
        #[pallet::constant]
        type MaxTimelockDuration: Get<BlockNumberFor<Self>>;

        /// Fee for creating an escrow
        #[pallet::constant]
        type EscrowFee: Get<Self::Balance>;
    }

    /// Storage for all escrows
    #[pallet::storage]
    #[pallet::getter(fn escrows)]
    pub type Escrows<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::EscrowId,
        Escrow<T>,
        OptionQuery,
    >;

    /// Storage for escrows by account (for efficient querying)
    #[pallet::storage]
    #[pallet::getter(fn account_escrows)]
    pub type AccountEscrows<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<T::EscrowId, T::MaxEscrowsPerAccount>,
        ValueQuery,
    >;

    /// Next available escrow ID
    #[pallet::storage]
    #[pallet::getter(fn next_escrow_id)]
    pub type NextEscrowId<T: Config> = StorageValue<_, T::EscrowId, ValueQuery>;

    /// Emergency pause flag
    #[pallet::storage]
    #[pallet::getter(fn emergency_paused)]
    pub type EmergencyPaused<T: Config> = StorageValue<_, bool, ValueQuery>;

    /// Events emitted by the pallet
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Escrow created successfully
        EscrowCreated {
            escrow_id: T::EscrowId,
            creator: T::AccountId,
            beneficiary: T::AccountId,
            asset: AssetInfo<T::AssetId>,
            amount: T::Balance,
            timelock: BlockNumberFor<T>,
        },

        /// Escrow funded and activated
        EscrowFunded {
            escrow_id: T::EscrowId,
            funder: T::AccountId,
        },

        /// Escrow completed successfully
        EscrowCompleted {
            escrow_id: T::EscrowId,
            beneficiary: T::AccountId,
            secret: Vec<u8>,
        },

        /// Escrow cancelled or expired
        EscrowCancelled {
            escrow_id: T::EscrowId,
            canceller: T::AccountId,
            reason: Vec<u8>,
        },

        /// Emergency pause activated
        EmergencyPauseActivated {
            activator: T::AccountId,
        },

        /// Emergency pause deactivated
        EmergencyPauseDeactivated {
            deactivator: T::AccountId,
        },
    }

    /// Errors that can occur in the pallet
    #[pallet::error]
    pub enum Error<T> {
        /// Escrow not found
        EscrowNotFound,
        /// Invalid escrow state for the requested operation
        InvalidEscrowState,
        /// Invalid hashlock provided
        InvalidHashlock,
        /// Invalid timelock duration
        InvalidTimelock,
        /// Timelock has expired
        TimelockExpired,
        /// Incorrect secret provided
        IncorrectSecret,
        /// Invalid secret format
        InvalidSecret,
        /// Not the escrow creator
        NotCreator,
        /// Not the escrow beneficiary
        NotBeneficiary,
        /// Insufficient balance for escrow
        InsufficientBalance,
        /// Asset not supported
        AssetNotSupported,
        /// Too many escrows for account
        TooManyEscrows,
        /// Invalid XCM route
        InvalidXcmRoute,
        /// XCM execution failed
        XcmExecutionFailed,
        /// Emergency pause is active
        EmergencyPaused,
        /// Operation not allowed during emergency pause
        OperationNotAllowed,
        /// Invalid metadata
        InvalidMetadata,
        /// Arithmetic overflow
        ArithmeticOverflow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new escrow with hashlock and timelock
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::create_escrow())]
        pub fn create_escrow(
            origin: OriginFor<T>,
            beneficiary: T::AccountId,
            asset: AssetInfo<T::AssetId>,
            amount: T::Balance,
            hashlock: Vec<u8>,
            timelock_duration: BlockNumberFor<T>,
            metadata: Vec<u8>,
            xcm_route: Option<XcmRoute>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Check emergency pause
            ensure!(!Self::emergency_paused(), Error::<T>::EmergencyPaused);

            // Validate inputs
            ensure!(
                !hashlock.is_empty() && hashlock.len() <= MAX_HASHLOCK_SIZE as usize,
                Error::<T>::InvalidHashlock
            );
            ensure!(
                timelock_duration >= T::MinTimelockDuration::get() &&
                timelock_duration <= T::MaxTimelockDuration::get(),
                Error::<T>::InvalidTimelock
            );
            ensure!(
                metadata.len() <= MAX_METADATA_SIZE as usize,
                Error::<T>::InvalidMetadata
            );

            // Check escrow limit
            let account_escrows = Self::account_escrows(&who);
            ensure!(
                account_escrows.len() < T::MaxEscrowsPerAccount::get() as usize,
                Error::<T>::TooManyEscrows
            );

            // Generate unique escrow ID
            let escrow_id = Self::next_escrow_id();
            let next_id = escrow_id.checked_add(&T::EscrowId::from(1u64)).ok_or(Error::<T>::ArithmeticOverflow)?;
            
            let current_block = frame_system::Pallet::<T>::block_number();
            let timelock = current_block.saturating_add(timelock_duration);

            // Create escrow
            let escrow = Escrow {
                id: escrow_id,
                creator: who.clone(),
                beneficiary: beneficiary.clone(),
                asset: asset.clone(),
                amount,
                hashlock: hashlock.try_into().map_err(|_| Error::<T>::InvalidHashlock)?,
                timelock,
                state: EscrowState::Created,
                metadata: metadata.try_into().map_err(|_| Error::<T>::InvalidMetadata)?,
                xcm_route,
                created_at: current_block,
                updated_at: current_block,
            };

            // Store escrow
            Escrows::<T>::insert(&escrow_id, &escrow);
            
            // Update account escrows
            AccountEscrows::<T>::try_mutate(&who, |escrows| {
                escrows.try_push(escrow_id).map_err(|_| Error::<T>::TooManyEscrows)
            })?;

            // Update next escrow ID
            NextEscrowId::<T>::put(next_id);

            // Emit event
            Self::deposit_event(Event::EscrowCreated {
                escrow_id,
                creator: who,
                beneficiary,
                asset,
                amount,
                timelock,
            });

            Ok(())
        }

        /// Fund an existing escrow
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::fund_escrow())]
        pub fn fund_escrow(
            origin: OriginFor<T>,
            escrow_id: T::EscrowId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Check emergency pause
            ensure!(!Self::emergency_paused(), Error::<T>::EmergencyPaused);

            // Get and validate escrow
            let mut escrow = Self::escrows(&escrow_id).ok_or(Error::<T>::EscrowNotFound)?;
            ensure!(escrow.state == EscrowState::Created, Error::<T>::InvalidEscrowState);
            ensure!(escrow.creator == who, Error::<T>::NotCreator);

            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(current_block < escrow.timelock, Error::<T>::TimelockExpired);

            // Transfer funds to escrow account
            let escrow_account = Self::escrow_account(&escrow_id);
            
            match &escrow.asset {
                AssetInfo::Native => {
                    T::Currency::transfer(&who, &escrow_account, escrow.amount, ExistenceRequirement::AllowDeath)?;
                },
                AssetInfo::Asset(asset_id) => {
                    T::Assets::transfer(*asset_id, &who, &escrow_account, escrow.amount, Preservation::Expendable)?;
                },
                AssetInfo::Nft(_, _) => {
                    // Future NFT support
                    return Err(Error::<T>::AssetNotSupported.into());
                },
            }

            // Update escrow state
            escrow.state = EscrowState::Active;
            escrow.updated_at = current_block;
            Escrows::<T>::insert(&escrow_id, &escrow);

            // Emit event
            Self::deposit_event(Event::EscrowFunded {
                escrow_id,
                funder: who,
            });

            Ok(())
        }

        /// Complete an escrow by providing the secret
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::complete_escrow())]
        pub fn complete_escrow(
            origin: OriginFor<T>,
            escrow_id: T::EscrowId,
            secret: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Check emergency pause
            ensure!(!Self::emergency_paused(), Error::<T>::EmergencyPaused);

            // Get and validate escrow
            let mut escrow = Self::escrows(&escrow_id).ok_or(Error::<T>::EscrowNotFound)?;
            ensure!(escrow.state == EscrowState::Active, Error::<T>::InvalidEscrowState);
            ensure!(escrow.beneficiary == who, Error::<T>::NotBeneficiary);

            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(current_block < escrow.timelock, Error::<T>::TimelockExpired);

            // Verify secret against hashlock
            let secret_hash = sp_core::hashing::sha2_256(&secret);
            ensure!(secret_hash.to_vec() == escrow.hashlock.to_vec(), Error::<T>::IncorrectSecret);

            // Transfer funds to beneficiary
            let escrow_account = Self::escrow_account(&escrow_id);
            
            match &escrow.asset {
                AssetInfo::Native => {
                    T::Currency::transfer(&escrow_account, &who, escrow.amount, ExistenceRequirement::AllowDeath)?;
                },
                AssetInfo::Asset(asset_id) => {
                    T::Assets::transfer(*asset_id, &escrow_account, &who, escrow.amount, Preservation::Expendable)?;
                },
                AssetInfo::Nft(_, _) => {
                    // Future NFT support
                    return Err(Error::<T>::AssetNotSupported.into());
                },
            }

            // Update escrow state
            escrow.state = EscrowState::Completed;
            escrow.updated_at = current_block;
            Escrows::<T>::insert(&escrow_id, &escrow);

            // Emit event
            Self::deposit_event(Event::EscrowCompleted {
                escrow_id,
                beneficiary: who,
                secret,
            });

            Ok(())
        }

        /// Cancel an expired or invalid escrow
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::cancel_escrow())]
        pub fn cancel_escrow(
            origin: OriginFor<T>,
            escrow_id: T::EscrowId,
            reason: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Get and validate escrow
            let mut escrow = Self::escrows(&escrow_id).ok_or(Error::<T>::EscrowNotFound)?;
            ensure!(
                escrow.state == EscrowState::Active || escrow.state == EscrowState::Created,
                Error::<T>::InvalidEscrowState
            );

            let current_block = frame_system::Pallet::<T>::block_number();
            
            // Check permissions: creator can cancel anytime, others only after timelock
            if who != escrow.creator {
                ensure!(current_block >= escrow.timelock, Error::<T>::InvalidTimelock);
            }

            // Refund if escrow is active
            if escrow.state == EscrowState::Active {
                let escrow_account = Self::escrow_account(&escrow_id);
                
                match &escrow.asset {
                    AssetInfo::Native => {
                        T::Currency::transfer(&escrow_account, &escrow.creator, escrow.amount, ExistenceRequirement::AllowDeath)?;
                    },
                    AssetInfo::Asset(asset_id) => {
                        T::Assets::transfer(*asset_id, &escrow_account, &escrow.creator, escrow.amount, Preservation::Expendable)?;
                    },
                    AssetInfo::Nft(_, _) => {
                        // Future NFT support
                        return Err(Error::<T>::AssetNotSupported.into());
                    },
                }
            }

            // Update escrow state
            escrow.state = EscrowState::Cancelled;
            escrow.updated_at = current_block;
            Escrows::<T>::insert(&escrow_id, &escrow);

            // Emit event
            Self::deposit_event(Event::EscrowCancelled {
                escrow_id,
                canceller: who,
                reason,
            });

            Ok(())
        }

        /// Emergency pause for security
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::emergency_pause())]
        pub fn emergency_pause(origin: OriginFor<T>) -> DispatchResult {
            // Try signed first, then root
            let who = if let Ok(signed) = ensure_signed(origin.clone()) {
                signed
            } else {
                ensure_root(origin)?;
                // Get the first account from the pallet for root calls
                T::PalletId::get().into_account_truncating()
            };

            EmergencyPaused::<T>::put(true);

            Self::deposit_event(Event::EmergencyPauseActivated {
                activator: who,
            });

            Ok(())
        }

        /// Deactivate emergency pause
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::emergency_unpause())]
        pub fn emergency_unpause(origin: OriginFor<T>) -> DispatchResult {
            // Allow either signed accounts or root
            let who = if let Ok(who) = ensure_signed(origin.clone()) {
                who
            } else {
                ensure_root(origin)?;
                // For root calls, we'll use a dummy account ID (this won't be stored anyway)
                Self::account_id()
            };

            EmergencyPaused::<T>::put(false);

            Self::deposit_event(Event::EmergencyPauseDeactivated {
                deactivator: who,
            });

            Ok(())
        }
    }

    // Helper methods
    impl<T: Config> Pallet<T> {
        /// Get the pallet's account ID
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }

        /// Generate deterministic account ID for escrow
        pub fn escrow_account(escrow_id: &T::EscrowId) -> T::AccountId {
            T::PalletId::get().into_sub_account_truncating(escrow_id)
        }

        /// Get escrow by ID
        pub fn get_escrow(escrow_id: &T::EscrowId) -> Option<Escrow<T>> {
            Self::escrows(escrow_id)
        }

        /// Get escrows for an account
        pub fn get_account_escrows(account: &T::AccountId) -> Vec<T::EscrowId> {
            Self::account_escrows(account).into_inner()
        }

        /// Check if timelock has expired
        pub fn is_timelock_expired(escrow_id: &T::EscrowId) -> bool {
            if let Some(escrow) = Self::escrows(escrow_id) {
                let current_block = frame_system::Pallet::<T>::block_number();
                current_block >= escrow.timelock
            } else {
                false
            }
        }

        /// Verify hashlock secret
        pub fn verify_secret(escrow_id: &T::EscrowId, secret: &[u8]) -> bool {
            if let Some(escrow) = Self::escrows(escrow_id) {
                let secret_hash = sp_core::hashing::sha2_256(secret);
                secret_hash.to_vec() == escrow.hashlock.to_vec()
            } else {
                false
            }
        }
    }
}
