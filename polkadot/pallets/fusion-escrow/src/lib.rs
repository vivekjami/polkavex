//! # Polkavex Fusion Escrow Pallet
//! 
//! A sophisticated cross-chain escrow pallet that implements Hash Time Locked Contracts (HTLC) 
//! with XCM integration for seamless parachain routing. This pallet provides the Polkadot-side 
//! infrastructure for Polkavex's cross-chain bridge, enabling secure atomic swaps between 
//! Ethereum and Polkadot ecosystems.
//!
//! ## Features
//!
//! - **Multi-Asset Support**: Native DOT, parachain tokens, and NFTs
//! - **Hash Time Locked Contracts**: Secure atomic swaps with secret-based completion
//! - **XCM Integration**: Cross-parachain asset routing (e.g., to Acala for yield optimization)
//! - **1inch Fusion+ Compatibility**: Mirrors Ethereum-side contract functionality
//! - **Gas Optimization**: Efficient storage patterns and execution weights
//! - **Security**: Comprehensive validation and emergency controls
//!
//! ## Usage
//!
//! ```rust,ignore
//! // Create an escrow with a secret hash and timelock
//! FusionEscrow::create_escrow(
//!     origin,
//!     secret_hash,
//!     timelock_block,
//!     taker_account,
//!     asset_id,
//!     amount,
//!     dest_parachain // Optional XCM routing
//! )?;
//!
//! // Complete escrow by revealing the secret
//! FusionEscrow::complete_escrow(origin, escrow_id, secret)?;
//!
//! // Cancel after timelock expires
//! FusionEscrow::cancel_escrow(origin, escrow_id)?;
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;

use frame_support::{
    dispatch::{DispatchResult, DispatchError},
    pallet_prelude::*,
    traits::{
        tokens::{fungibles::{Inspect, Mutate, Transfer}, Preservation},
        Get, UnixTime,
    },
    PalletId,
};
use frame_system::pallet_prelude::*;
use sp_core::crypto::UncheckedFrom;
use sp_runtime::{
    traits::{AccountIdConversion, BlakeTwo256, Hash, Saturating, Zero},
    ArithmeticError,
};
use sp_std::{vec::Vec, collections::btree_map::BTreeMap};
// XCM temporarily disabled for initial build
// use xcm::prelude::*;

/// Pallet ID for generating sovereign accounts
const PALLET_ID: PalletId = PalletId(*b"plkv/esc");

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Configuration trait for the pallet
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The currency used for native balance operations
        type Currency: Inspect<Self::AccountId> + Mutate<Self::AccountId> + Transfer<Self::AccountId>;

        /// Multi-asset support for parachain tokens
        type Assets: Inspect<Self::AccountId, AssetId = u32, Balance = u128>
            + Mutate<Self::AccountId, AssetId = u32, Balance = u128>
            + Transfer<Self::AccountId, AssetId = u32, Balance = u128>;

        /// XCM executor for cross-chain operations (temporarily disabled)
        // type XcmExecutor: ExecuteXcm<Self::RuntimeCall>;

        /// Weight information for extrinsics
        type WeightInfo: WeightInfo;

        /// Maximum number of active escrows per account
        #[pallet::constant]
        type MaxEscrowsPerAccount: Get<u32>;

        /// Minimum timelock duration (in blocks)
        #[pallet::constant]
        type MinTimelock: Get<Self::BlockNumber>;

        /// Maximum timelock duration (in blocks)
        #[pallet::constant]
        type MaxTimelock: Get<Self::BlockNumber>;

        /// Time provider for getting current timestamp
        type TimeProvider: UnixTime;
    }

    /// Asset types supported by the escrow system
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum AssetType {
        /// Native DOT currency
        Native,
        /// Parachain asset by ID
        Asset(u32),
        /// NFT (future expansion)
        Nft(u32, u32), // collection_id, item_id
    }

    /// Escrow states throughout its lifecycle
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum EscrowState {
        /// Created but not yet funded
        Created,
        /// Funded and active
        Active,
        /// Successfully completed with secret reveal
        Completed,
        /// Cancelled due to timeout or other reasons
        Cancelled,
    }

    /// Comprehensive escrow details
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct EscrowDetails<AccountId, BlockNumber> {
        /// Hash of the secret required for completion
        pub secret_hash: [u8; 32],
        /// Account that created the escrow
        pub maker: AccountId,
        /// Account designated to receive the assets
        pub taker: AccountId,
        /// Block number when the escrow expires
        pub timelock: BlockNumber,
        /// Type and amount of assets escrowed
        pub asset_type: AssetType,
        /// Amount of assets (in smallest unit)
        pub amount: u128,
        /// Current state of the escrow
        pub state: EscrowState,
        /// Optional XCM destination for cross-parachain routing (temporarily disabled)
        pub xcm_destination: Option<u32>,  // Placeholder for MultiLocation
        /// Block when escrow was created
        pub created_block: BlockNumber,
        /// Optional metadata for additional context
        pub metadata: BoundedVec<u8, ConstU32<256>>,
    }

    /// Events emitted by the pallet
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// An escrow was successfully created
        EscrowCreated {
            escrow_id: u32,
            maker: T::AccountId,
            taker: T::AccountId,
            secret_hash: [u8; 32],
            timelock: T::BlockNumber,
            asset_type: AssetType,
            amount: u128,
        },
        /// Assets were deposited into an escrow
        EscrowFunded {
            escrow_id: u32,
            asset_type: AssetType,
            amount: u128,
        },
        /// An escrow was completed successfully
        EscrowCompleted {
            escrow_id: u32,
            taker: T::AccountId,
            secret: [u8; 32],
        },
        /// An escrow was cancelled
        EscrowCancelled {
            escrow_id: u32,
            maker: T::AccountId,
            reason: Vec<u8>,
        },
        /// XCM transfer was initiated (temporarily disabled)
        XcmTransferInitiated {
            escrow_id: u32,
            destination: u32,  // Placeholder for MultiLocation
            asset_type: AssetType,
            amount: u128,
        },
        /// Emergency pause toggled
        EmergencyPauseToggled {
            paused: bool,
        },
    }

    /// Errors that can occur in pallet operations
    #[pallet::error]
    pub enum Error<T> {
        /// Escrow with the given ID does not exist
        EscrowNotFound,
        /// Escrow is not in the expected state for this operation
        InvalidEscrowState,
        /// The provided secret does not match the hash
        InvalidSecret,
        /// The timelock has not yet expired
        TimelockNotExpired,
        /// The timelock has already expired
        TimelockExpired,
        /// Insufficient balance for the operation
        InsufficientBalance,
        /// The account is not authorized for this operation
        NotAuthorized,
        /// Invalid timelock duration
        InvalidTimelock,
        /// Maximum number of escrows per account exceeded
        TooManyEscrows,
        /// Invalid asset type or ID
        InvalidAsset,
        /// XCM execution failed
        XcmExecutionFailed,
        /// Arithmetic overflow
        Overflow,
        /// The pallet is currently paused
        PalletPaused,
        /// Invalid taker address
        InvalidTaker,
        /// Duplicate secret hash detected
        DuplicateSecretHash,
    }

    /// Storage for individual escrows
    #[pallet::storage]
    #[pallet::getter(fn escrows)]
    pub type Escrows<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32,
        EscrowDetails<T::AccountId, T::BlockNumber>,
        OptionQuery,
    >;

    /// Counter for generating unique escrow IDs
    #[pallet::storage]
    #[pallet::getter(fn next_escrow_id)]
    pub type NextEscrowId<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Index of escrows by secret hash (for uniqueness checking)
    #[pallet::storage]
    #[pallet::getter(fn escrows_by_secret)]
    pub type EscrowsBySecret<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        [u8; 32],
        u32,
        OptionQuery,
    >;

    /// Index of escrows by maker account
    #[pallet::storage]
    #[pallet::getter(fn escrows_by_maker)]
    pub type EscrowsByMaker<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u32, T::MaxEscrowsPerAccount>,
        ValueQuery,
    >;

    /// Index of escrows by taker account
    #[pallet::storage]
    #[pallet::getter(fn escrows_by_taker)]
    pub type EscrowsByTaker<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u32, T::MaxEscrowsPerAccount>,
        ValueQuery,
    >;

    /// Emergency pause flag
    #[pallet::storage]
    #[pallet::getter(fn is_paused)]
    pub type IsPaused<T: Config> = StorageValue<_, bool, ValueQuery>;

    /// Genesis configuration
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub phantom: sp_std::marker::PhantomData<T>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                phantom: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            // Initialize the next escrow ID to 1
            <NextEscrowId<T>>::put(1u32);
        }
    }

    /// Dispatchable functions (extrinsics)
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new escrow with the specified parameters
        ///
        /// Parameters:
        /// - `secret_hash`: Blake2-256 hash of the secret required for completion
        /// - `timelock`: Block number when the escrow expires
        /// - `taker`: Account designated to receive the assets
        /// - `asset_type`: Type of asset being escrowed
        /// - `amount`: Amount of assets in smallest unit
        /// - `xcm_destination`: Optional cross-parachain destination
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::create_escrow())]
        pub fn create_escrow(
            origin: OriginFor<T>,
            secret_hash: [u8; 32],
            timelock: T::BlockNumber,
            taker: T::AccountId,
            asset_type: AssetType,
            amount: u128,
            xcm_destination: Option<u32>,  // Placeholder for MultiLocation
            metadata: BoundedVec<u8, ConstU32<256>>,
        ) -> DispatchResult {
            let maker = ensure_signed(origin)?;
            
            // Check if pallet is paused
            ensure!(!Self::is_paused(), Error::<T>::PalletPaused);
            
            // Validate timelock
            let current_block = <frame_system::Pallet<T>>::block_number();
            let min_timelock = current_block + T::MinTimelock::get();
            let max_timelock = current_block + T::MaxTimelock::get();
            
            ensure!(timelock >= min_timelock, Error::<T>::InvalidTimelock);
            ensure!(timelock <= max_timelock, Error::<T>::InvalidTimelock);
            
            // Ensure taker is different from maker
            ensure!(maker != taker, Error::<T>::InvalidTaker);
            
            // Check for duplicate secret hash
            ensure!(!EscrowsBySecret::<T>::contains_key(&secret_hash), Error::<T>::DuplicateSecretHash);
            
            // Check escrow limits per account
            let maker_escrows = Self::escrows_by_maker(&maker);
            ensure!(
                maker_escrows.len() < T::MaxEscrowsPerAccount::get() as usize,
                Error::<T>::TooManyEscrows
            );
            
            // Validate amount is not zero
            ensure!(!amount.is_zero(), Error::<T>::InvalidAsset);
            
            // Generate unique escrow ID
            let escrow_id = Self::next_escrow_id();
            let next_id = escrow_id.saturating_add(1);
            <NextEscrowId<T>>::put(next_id);
            
            // Create escrow details
            let escrow = EscrowDetails {
                secret_hash,
                maker: maker.clone(),
                taker: taker.clone(),
                timelock,
                asset_type: asset_type.clone(),
                amount,
                state: EscrowState::Created,
                xcm_destination: xcm_destination.clone(),
                created_block: current_block,
                metadata,
            };
            
            // Store the escrow
            <Escrows<T>>::insert(&escrow_id, &escrow);
            <EscrowsBySecret<T>>::insert(&secret_hash, &escrow_id);
            
            // Update maker's escrow list
            <EscrowsByMaker<T>>::try_mutate(&maker, |escrows| {
                escrows.try_push(escrow_id)
            }).map_err(|_| Error::<T>::TooManyEscrows)?;
            
            // Update taker's escrow list
            <EscrowsByTaker<T>>::try_mutate(&taker, |escrows| {
                escrows.try_push(escrow_id)
            }).map_err(|_| Error::<T>::TooManyEscrows)?;
            
            // Emit event
            Self::deposit_event(Event::EscrowCreated {
                escrow_id,
                maker,
                taker,
                secret_hash,
                timelock,
                asset_type,
                amount,
            });
            
            Ok(())
        }

        /// Fund an escrow by transferring assets to the pallet account
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::fund_escrow())]
        pub fn fund_escrow(
            origin: OriginFor<T>,
            escrow_id: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Check if pallet is paused
            ensure!(!Self::is_paused(), Error::<T>::PalletPaused);
            
            // Get escrow details
            let mut escrow = Self::escrows(escrow_id).ok_or(Error::<T>::EscrowNotFound)?;
            
            // Verify the caller is the maker
            ensure!(who == escrow.maker, Error::<T>::NotAuthorized);
            
            // Verify escrow is in Created state
            ensure!(escrow.state == EscrowState::Created, Error::<T>::InvalidEscrowState);
            
            // Check timelock hasn't expired
            let current_block = <frame_system::Pallet<T>>::block_number();
            ensure!(current_block < escrow.timelock, Error::<T>::TimelockExpired);
            
            // Get the pallet's sovereign account
            let pallet_account = Self::account_id();
            
            // Transfer assets based on type
            match &escrow.asset_type {
                AssetType::Native => {
                    T::Currency::transfer(
                        &who,
                        &pallet_account,
                        escrow.amount,
                        Preservation::Preserve,
                    )?;
                },
                AssetType::Asset(asset_id) => {
                    T::Assets::transfer(
                        *asset_id,
                        &who,
                        &pallet_account,
                        escrow.amount,
                        Preservation::Preserve,
                    )?;
                },
                AssetType::Nft(collection_id, item_id) => {
                    // NFT transfer logic would go here
                    // For now, we'll return an error as NFTs need specialized handling
                    return Err(Error::<T>::InvalidAsset.into());
                },
            }
            
            // Update escrow state to Active
            escrow.state = EscrowState::Active;
            <Escrows<T>>::insert(&escrow_id, &escrow);
            
            // Emit event
            Self::deposit_event(Event::EscrowFunded {
                escrow_id,
                asset_type: escrow.asset_type,
                amount: escrow.amount,
            });
            
            Ok(())
        }

        /// Complete an escrow by providing the secret
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::complete_escrow())]
        pub fn complete_escrow(
            origin: OriginFor<T>,
            escrow_id: u32,
            secret: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Check if pallet is paused
            ensure!(!Self::is_paused(), Error::<T>::PalletPaused);
            
            // Get escrow details
            let mut escrow = Self::escrows(escrow_id).ok_or(Error::<T>::EscrowNotFound)?;
            
            // Verify escrow is in Active state
            ensure!(escrow.state == EscrowState::Active, Error::<T>::InvalidEscrowState);
            
            // Check timelock hasn't expired
            let current_block = <frame_system::Pallet<T>>::block_number();
            ensure!(current_block < escrow.timelock, Error::<T>::TimelockExpired);
            
            // Verify the secret hash
            let computed_hash = BlakeTwo256::hash(&secret);
            ensure!(computed_hash.as_ref() == &escrow.secret_hash, Error::<T>::InvalidSecret);
            
            // Get the pallet's sovereign account
            let pallet_account = Self::account_id();
            
            // Transfer assets to the taker
            match &escrow.asset_type {
                AssetType::Native => {
                    T::Currency::transfer(
                        &pallet_account,
                        &escrow.taker,
                        escrow.amount,
                        Preservation::Expendable,
                    )?;
                },
                AssetType::Asset(asset_id) => {
                    T::Assets::transfer(
                        *asset_id,
                        &pallet_account,
                        &escrow.taker,
                        escrow.amount,
                        Preservation::Expendable,
                    )?;
                },
                AssetType::Nft(collection_id, item_id) => {
                    // NFT transfer logic would go here
                    return Err(Error::<T>::InvalidAsset.into());
                },
            }
            
            // Update escrow state to Completed
            escrow.state = EscrowState::Completed;
            <Escrows<T>>::insert(&escrow_id, &escrow);
            
            // Emit event
            Self::deposit_event(Event::EscrowCompleted {
                escrow_id,
                taker: escrow.taker,
                secret,
            });
            
            Ok(())
        }

        /// Cancel an escrow and refund the maker (only after timelock expires)
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::cancel_escrow())]
        pub fn cancel_escrow(
            origin: OriginFor<T>,
            escrow_id: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Check if pallet is paused
            ensure!(!Self::is_paused(), Error::<T>::PalletPaused);
            
            // Get escrow details
            let mut escrow = Self::escrows(escrow_id).ok_or(Error::<T>::EscrowNotFound)?;
            
            // Verify the caller is the maker
            ensure!(who == escrow.maker, Error::<T>::NotAuthorized);
            
            // Verify escrow is in Active state
            ensure!(escrow.state == EscrowState::Active, Error::<T>::InvalidEscrowState);
            
            // Check timelock has expired
            let current_block = <frame_system::Pallet<T>>::block_number();
            ensure!(current_block >= escrow.timelock, Error::<T>::TimelockNotExpired);
            
            // Get the pallet's sovereign account
            let pallet_account = Self::account_id();
            
            // Refund assets to the maker
            match &escrow.asset_type {
                AssetType::Native => {
                    T::Currency::transfer(
                        &pallet_account,
                        &escrow.maker,
                        escrow.amount,
                        Preservation::Expendable,
                    )?;
                },
                AssetType::Asset(asset_id) => {
                    T::Assets::transfer(
                        *asset_id,
                        &pallet_account,
                        &escrow.maker,
                        escrow.amount,
                        Preservation::Expendable,
                    )?;
                },
                AssetType::Nft(collection_id, item_id) => {
                    // NFT transfer logic would go here
                    return Err(Error::<T>::InvalidAsset.into());
                },
            }
            
            // Update escrow state to Cancelled
            escrow.state = EscrowState::Cancelled;
            <Escrows<T>>::insert(&escrow_id, &escrow);
            
            // Emit event
            Self::deposit_event(Event::EscrowCancelled {
                escrow_id,
                maker: escrow.maker,
                reason: b"Timelock expired".to_vec(),
            });
            
            Ok(())
        }

        /// Cancel an escrow before funding (emergency function for maker)
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::cancel_before_funding())]
        pub fn cancel_before_funding(
            origin: OriginFor<T>,
            escrow_id: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Check if pallet is paused
            ensure!(!Self::is_paused(), Error::<T>::PalletPaused);
            
            // Get escrow details
            let mut escrow = Self::escrows(escrow_id).ok_or(Error::<T>::EscrowNotFound)?;
            
            // Verify the caller is the maker
            ensure!(who == escrow.maker, Error::<T>::NotAuthorized);
            
            // Verify escrow is in Created state (not yet funded)
            ensure!(escrow.state == EscrowState::Created, Error::<T>::InvalidEscrowState);
            
            // Update escrow state to Cancelled
            escrow.state = EscrowState::Cancelled;
            <Escrows<T>>::insert(&escrow_id, &escrow);
            
            // Remove from secret hash index
            <EscrowsBySecret<T>>::remove(&escrow.secret_hash);
            
            // Emit event
            Self::deposit_event(Event::EscrowCancelled {
                escrow_id,
                maker: escrow.maker,
                reason: b"Cancelled before funding".to_vec(),
            });
            
            Ok(())
        }

        /// Emergency pause toggle (root only)
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::toggle_pause())]
        pub fn toggle_pause(
            origin: OriginFor<T>,
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            let current_state = Self::is_paused();
            let new_state = !current_state;
            
            <IsPaused<T>>::put(new_state);
            
            Self::deposit_event(Event::EmergencyPauseToggled {
                paused: new_state,
            });
            
            Ok(())
        }
    }

    // Helper functions
    impl<T: Config> Pallet<T> {
        /// Get the pallet's sovereign account ID
        pub fn account_id() -> T::AccountId {
            PALLET_ID.into_account_truncating()
        }

        /// Get escrow details by ID
        pub fn get_escrow(escrow_id: u32) -> Option<EscrowDetails<T::AccountId, T::BlockNumber>> {
            Self::escrows(escrow_id)
        }

        /// Get escrow ID by secret hash
        pub fn get_escrow_by_secret(secret_hash: &[u8; 32]) -> Option<u32> {
            Self::escrows_by_secret(secret_hash)
        }

        /// Check if an escrow is active
        pub fn is_escrow_active(escrow_id: u32) -> bool {
            if let Some(escrow) = Self::escrows(escrow_id) {
                escrow.state == EscrowState::Active
            } else {
                false
            }
        }

        /// Get time remaining for an escrow
        pub fn get_time_remaining(escrow_id: u32) -> Option<T::BlockNumber> {
            if let Some(escrow) = Self::escrows(escrow_id) {
                let current_block = <frame_system::Pallet<T>>::block_number();
                if current_block < escrow.timelock {
                    Some(escrow.timelock - current_block)
                } else {
                    Some(Zero::zero())
                }
            } else {
                None
            }
        }

        /// Get all escrows for a maker
        pub fn get_escrows_by_maker(maker: &T::AccountId) -> Vec<u32> {
            Self::escrows_by_maker(maker).into_inner()
        }

        /// Get all escrows for a taker  
        pub fn get_escrows_by_taker(taker: &T::AccountId) -> Vec<u32> {
            Self::escrows_by_taker(taker).into_inner()
        }
    }
}
