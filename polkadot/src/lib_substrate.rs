// Polkavex Substrate Pallet - Basic Structure
// This will be expanded in Day 2 with full XCM integration

use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
};
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn escrows)]
    pub type Escrows<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        [u8; 32], // secret hash
        (T::AccountId, T::BlockNumber, u32), // (creator, timelock, amount)
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Escrow created [secret_hash, creator, timelock, amount]
        EscrowCreated([u8; 32], T::AccountId, T::BlockNumber, u32),
        /// Escrow resolved [secret_hash, resolver]  
        EscrowResolved([u8; 32], T::AccountId),
        /// Escrow cancelled [secret_hash]
        EscrowCancelled([u8; 32]),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Escrow already exists
        EscrowAlreadyExists,
        /// Escrow not found
        EscrowNotFound,
        /// Timelock has not expired
        TimelockNotExpired,
        /// Invalid secret provided
        InvalidSecret,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new escrow with hashlock and timelock
        #[pallet::weight(10_000)]
        pub fn create_escrow(
            origin: OriginFor<T>,
            secret_hash: [u8; 32],
            timelock: T::BlockNumber,
            amount: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(!Escrows::<T>::contains_key(&secret_hash), Error::<T>::EscrowAlreadyExists);

            Escrows::<T>::insert(&secret_hash, (&who, timelock, amount));

            Self::deposit_event(Event::EscrowCreated(secret_hash, who, timelock, amount));

            Ok(())
        }

        /// Resolve escrow with secret (placeholder for XCM integration)
        #[pallet::weight(10_000)]
        pub fn resolve_escrow(
            origin: OriginFor<T>,
            secret: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Simple hash check (will be enhanced with proper hashing)
            let secret_hash = sp_runtime::traits::BlakeTwo256::hash(&secret);
            let hash_bytes: [u8; 32] = secret_hash.into();

            ensure!(Escrows::<T>::contains_key(&hash_bytes), Error::<T>::EscrowNotFound);

            Escrows::<T>::remove(&hash_bytes);

            Self::deposit_event(Event::EscrowResolved(hash_bytes, who));

            Ok(())
        }

        /// Cancel escrow after timelock expires
        #[pallet::weight(10_000)]
        pub fn cancel_escrow(
            origin: OriginFor<T>,
            secret_hash: [u8; 32],
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            let (_, timelock, _) = Escrows::<T>::get(&secret_hash)
                .ok_or(Error::<T>::EscrowNotFound)?;

            let current_block = <frame_system::Pallet<T>>::block_number();
            ensure!(current_block > timelock, Error::<T>::TimelockNotExpired);

            Escrows::<T>::remove(&secret_hash);

            Self::deposit_event(Event::EscrowCancelled(secret_hash));

            Ok(())
        }
    }
}

// Basic tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_escrow_test() {
        // TODO: Add proper test framework in Day 2
        println!("Polkavex pallet structure created successfully!");
    }
}
