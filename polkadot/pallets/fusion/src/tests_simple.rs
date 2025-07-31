//! Tests for the fusion pallet

use crate::{mock::*, Error, Event, AssetInfo, EscrowState, XcmRoute};
use frame_support::{
    assert_err, assert_ok, assert_noop,
    traits::Hooks,
    BoundedVec,
};
use sp_core::H256;

#[test]
fn create_escrow_works() {
    new_test_ext().execute_with(|| {
        let beneficiary = 2u64;
        let asset = AssetInfo::Native;
        let amount = 1000u128;
        let hashlock = b"test_secret_hash".to_vec();
        let timelock_duration = 100u64;
        let metadata = b"test metadata".to_vec();
        let xcm_route = None;
        
        assert_ok!(Fusion::create_escrow(
            RuntimeOrigin::signed(1),
            beneficiary,
            asset,
            amount,
            hashlock,
            timelock_duration,
            metadata,
            xcm_route,
        ));
        
        // Check that the escrow was created
        let escrow = Fusion::get_escrow(&1u64).unwrap();
        assert_eq!(escrow.beneficiary, beneficiary);
        assert_eq!(escrow.amount, amount);
        assert_eq!(escrow.state, EscrowState::Created);
        
        // Check that the event was emitted
        let event = last_event();
        assert!(matches!(
            event,
            RuntimeEvent::Fusion(Event::EscrowCreated { .. })
        ));
    });
}

#[test]
fn fund_escrow_works() {
    new_test_ext().execute_with(|| {
        let beneficiary = 2u64;
        let asset = AssetInfo::Native;
        let amount = 1000u128;
        let hashlock = b"test_secret_hash".to_vec();
        let timelock_duration = 100u64;
        let metadata = b"test metadata".to_vec();
        let xcm_route = None;
        
        // First create an escrow
        assert_ok!(Fusion::create_escrow(
            RuntimeOrigin::signed(1),
            beneficiary,
            asset,
            amount,
            hashlock,
            timelock_duration,
            metadata,
            xcm_route,
        ));
        
        // Then fund it
        assert_ok!(Fusion::fund_escrow(RuntimeOrigin::signed(1), 1u64));
        
        // Check that the escrow state was updated
        let escrow = Fusion::get_escrow(&1u64).unwrap();
        assert_eq!(escrow.state, EscrowState::Active);
        
        // Check that the event was emitted
        let events = System::events();
        assert!(events.iter().any(|e| matches!(
            e.event,
            RuntimeEvent::Fusion(Event::EscrowFunded { .. })
        )));
    });
}

#[test]
fn complete_escrow_works() {
    new_test_ext().execute_with(|| {
        let beneficiary = 2u64;
        let asset = AssetInfo::Native;
        let amount = 1000u128;
        let secret = b"test_secret";
        let hashlock = sp_core::hashing::sha2_256(secret).to_vec();
        let timelock_duration = 100u64;
        let metadata = b"test metadata".to_vec();
        let xcm_route = None;
        
        // Create and fund escrow
        assert_ok!(Fusion::create_escrow(
            RuntimeOrigin::signed(1),
            beneficiary,
            asset,
            amount,
            hashlock,
            timelock_duration,
            metadata,
            xcm_route,
        ));
        
        assert_ok!(Fusion::fund_escrow(RuntimeOrigin::signed(1), 1u64));
        
        // Complete the escrow with the secret
        assert_ok!(Fusion::complete_escrow(
            RuntimeOrigin::signed(beneficiary),
            1u64,
            secret.to_vec(),
        ));
        
        // Check that the escrow state was updated
        let escrow = Fusion::get_escrow(&1u64).unwrap();
        assert_eq!(escrow.state, EscrowState::Completed);
        
        // Check that the event was emitted
        let events = System::events();
        assert!(events.iter().any(|e| matches!(
            e.event,
            RuntimeEvent::Fusion(Event::EscrowCompleted { .. })
        )));
    });
}

#[test]
fn cancel_escrow_works() {
    new_test_ext().execute_with(|| {
        let beneficiary = 2u64;
        let asset = AssetInfo::Native;
        let amount = 1000u128;
        let hashlock = b"test_secret_hash".to_vec();
        let timelock_duration = 10u64; // Short timelock
        let metadata = b"test metadata".to_vec();
        let xcm_route = None;
        
        // Create and fund escrow
        assert_ok!(Fusion::create_escrow(
            RuntimeOrigin::signed(1),
            beneficiary,
            asset,
            amount,
            hashlock,
            timelock_duration,
            metadata,
            xcm_route,
        ));
        
        assert_ok!(Fusion::fund_escrow(RuntimeOrigin::signed(1), 1u64));
        
        // Wait for timelock to expire
        run_to_block(20);
        
        // Cancel the escrow
        let reason = b"Timelock expired".to_vec();
        assert_ok!(Fusion::cancel_escrow(
            RuntimeOrigin::signed(1),
            1u64,
            reason,
        ));
        
        // Check that the escrow state was updated
        let escrow = Fusion::get_escrow(&1u64).unwrap();
        assert_eq!(escrow.state, EscrowState::Cancelled);
        
        // Check that the event was emitted
        let events = System::events();
        assert!(events.iter().any(|e| matches!(
            e.event,
            RuntimeEvent::Fusion(Event::EscrowCancelled { .. })
        )));
    });
}

#[test]
fn emergency_pause_works() {
    new_test_ext().execute_with(|| {
        // Pause the pallet
        assert_ok!(Fusion::emergency_pause(RuntimeOrigin::signed(1)));
        
        // Try to create escrow - should fail
        let beneficiary = 2u64;
        let asset = AssetInfo::Native;
        let amount = 1000u128;
        let hashlock = b"test_secret_hash".to_vec();
        let timelock_duration = 100u64;
        let metadata = b"test metadata".to_vec();
        let xcm_route = None;
        
        assert_noop!(
            Fusion::create_escrow(
                RuntimeOrigin::signed(1),
                beneficiary,
                asset,
                amount,
                hashlock,
                timelock_duration,
                metadata,
                xcm_route,
            ),
            Error::<Test>::EmergencyPaused
        );
        
        // Unpause the pallet
        assert_ok!(Fusion::emergency_unpause(RuntimeOrigin::signed(1)));
        
        // Now creation should work
        assert_ok!(Fusion::create_escrow(
            RuntimeOrigin::signed(1),
            beneficiary,
            asset,
            amount,
            hashlock,
            timelock_duration,
            metadata,
            xcm_route,
        ));
    });
}

#[test]
fn invalid_secret_fails() {
    new_test_ext().execute_with(|| {
        let beneficiary = 2u64;
        let asset = AssetInfo::Native;
        let amount = 1000u128;
        let secret = b"test_secret";
        let hashlock = sp_core::hashing::sha2_256(secret).to_vec();
        let timelock_duration = 100u64;
        let metadata = b"test metadata".to_vec();
        let xcm_route = None;
        
        // Create and fund escrow
        assert_ok!(Fusion::create_escrow(
            RuntimeOrigin::signed(1),
            beneficiary,
            asset,
            amount,
            hashlock,
            timelock_duration,
            metadata,
            xcm_route,
        ));
        
        assert_ok!(Fusion::fund_escrow(RuntimeOrigin::signed(1), 1u64));
        
        // Try to complete with wrong secret
        let wrong_secret = b"wrong_secret";
        assert_noop!(
            Fusion::complete_escrow(
                RuntimeOrigin::signed(beneficiary),
                1u64,
                wrong_secret.to_vec(),
            ),
            Error::<Test>::InvalidSecret
        );
    });
}
