//! Benchmarking for pallet-fusion-escrow

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{benchmarks, whitelisted_caller, impl_benchmark_test_suite};
use frame_support::{
    traits::{tokens::Preservation, Get},
    BoundedVec,
};
use frame_system::RawOrigin;
use sp_core::blake2_256;

const SEED: u32 = 0;

benchmarks! {
    create_escrow {
        let caller: T::AccountId = whitelisted_caller();
        let taker: T::AccountId = whitelisted_caller();
        let secret_hash = blake2_256(b"benchmark_secret");
        let timelock = 1000u32.into();
        let amount = 1000u128;
        let metadata = BoundedVec::try_from(b"benchmark".to_vec()).unwrap();
        
        // Ensure caller has sufficient balance
        T::Currency::mint_into(&caller, 10000u128)?;
    }: _(
        RawOrigin::Signed(caller),
        secret_hash,
        timelock,
        taker,
        AssetType::Native,
        amount,
        None,
        metadata
    )
    verify {
        assert_eq!(Pallet::<T>::next_escrow_id(), 2);
        assert!(Pallet::<T>::escrows(1).is_some());
    }

    fund_escrow {
        let caller: T::AccountId = whitelisted_caller();
        let taker: T::AccountId = whitelisted_caller();
        let secret_hash = blake2_256(b"benchmark_secret");
        let timelock = 1000u32.into();
        let amount = 1000u128;
        let metadata = BoundedVec::try_from(b"benchmark".to_vec()).unwrap();
        
        // Ensure caller has sufficient balance
        T::Currency::mint_into(&caller, 10000u128)?;
        
        // Create escrow first
        Pallet::<T>::create_escrow(
            RawOrigin::Signed(caller.clone()).into(),
            secret_hash,
            timelock,
            taker,
            AssetType::Native,
            amount,
            None,
            metadata
        )?;
    }: _(RawOrigin::Signed(caller), 1)
    verify {
        let escrow = Pallet::<T>::escrows(1).unwrap();
        assert_eq!(escrow.state, EscrowState::Active);
    }

    complete_escrow {
        let caller: T::AccountId = whitelisted_caller();
        let taker: T::AccountId = whitelisted_caller();
        let secret = b"benchmark_secret_1234567890123456";
        let secret_hash = blake2_256(secret);
        let timelock = 1000u32.into();
        let amount = 1000u128;
        let metadata = BoundedVec::try_from(b"benchmark".to_vec()).unwrap();
        
        // Ensure caller has sufficient balance
        T::Currency::mint_into(&caller, 10000u128)?;
        
        // Create and fund escrow
        Pallet::<T>::create_escrow(
            RawOrigin::Signed(caller.clone()).into(),
            secret_hash,
            timelock,
            taker.clone(),
            AssetType::Native,
            amount,
            None,
            metadata
        )?;
        
        Pallet::<T>::fund_escrow(
            RawOrigin::Signed(caller).into(),
            1
        )?;
    }: _(RawOrigin::Signed(taker), 1, *secret)
    verify {
        let escrow = Pallet::<T>::escrows(1).unwrap();
        assert_eq!(escrow.state, EscrowState::Completed);
    }

    cancel_escrow {
        let caller: T::AccountId = whitelisted_caller();
        let taker: T::AccountId = whitelisted_caller();
        let secret_hash = blake2_256(b"benchmark_secret");
        let timelock = 10u32.into();
        let amount = 1000u128;
        let metadata = BoundedVec::try_from(b"benchmark".to_vec()).unwrap();
        
        // Ensure caller has sufficient balance
        T::Currency::mint_into(&caller, 10000u128)?;
        
        // Create and fund escrow
        Pallet::<T>::create_escrow(
            RawOrigin::Signed(caller.clone()).into(),
            secret_hash,
            timelock,
            taker,
            AssetType::Native,
            amount,
            None,
            metadata
        )?;
        
        Pallet::<T>::fund_escrow(
            RawOrigin::Signed(caller.clone()).into(),
            1
        )?;
        
        // Move past timelock
        frame_system::Pallet::<T>::set_block_number(timelock + 1u32.into());
    }: _(RawOrigin::Signed(caller), 1)
    verify {
        let escrow = Pallet::<T>::escrows(1).unwrap();
        assert_eq!(escrow.state, EscrowState::Cancelled);
    }

    cancel_before_funding {
        let caller: T::AccountId = whitelisted_caller();
        let taker: T::AccountId = whitelisted_caller();
        let secret_hash = blake2_256(b"benchmark_secret");
        let timelock = 1000u32.into();
        let amount = 1000u128;
        let metadata = BoundedVec::try_from(b"benchmark".to_vec()).unwrap();
        
        // Create escrow (but don't fund)
        Pallet::<T>::create_escrow(
            RawOrigin::Signed(caller.clone()).into(),
            secret_hash,
            timelock,
            taker,
            AssetType::Native,
            amount,
            None,
            metadata
        )?;
    }: _(RawOrigin::Signed(caller), 1)
    verify {
        let escrow = Pallet::<T>::escrows(1).unwrap();
        assert_eq!(escrow.state, EscrowState::Cancelled);
    }

    toggle_pause {
    }: _(RawOrigin::Root)
    verify {
        assert_eq!(Pallet::<T>::is_paused(), true);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
