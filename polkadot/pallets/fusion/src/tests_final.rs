//! Simple tests for the fusion pallet

use crate::{mock::*, Error, Event, AssetInfo, EscrowState};
use frame_support::{
    assert_ok, assert_noop,
};

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
        // Create an escrow first
        assert_ok!(Fusion::create_escrow(
            RuntimeOrigin::signed(1),
            2u64,
            AssetInfo::Native,
            1000u128,
            b"test_secret_hash".to_vec(),
            100u64,
            b"test metadata".to_vec(),
            None,
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
        let secret = b"test_secret";
        let hashlock = sp_core::hashing::sha2_256(secret).to_vec();
        
        // Create and fund escrow
        assert_ok!(Fusion::create_escrow(
            RuntimeOrigin::signed(1),
            2u64,
            AssetInfo::Native,
            1000u128,
            hashlock,
            100u64,
            b"test metadata".to_vec(),
            None,
        ));
        
        assert_ok!(Fusion::fund_escrow(RuntimeOrigin::signed(1), 1u64));
        
        // Complete the escrow with the secret
        assert_ok!(Fusion::complete_escrow(
            RuntimeOrigin::signed(2),
            1u64,
            secret.to_vec(),
        ));
        
        // Check that the escrow state was updated
        let escrow = Fusion::get_escrow(&1u64).unwrap();
        assert_eq!(escrow.state, EscrowState::Completed);
    });
}

#[test]
fn cancel_escrow_works() {
    new_test_ext().execute_with(|| {
        // Create and fund escrow with short timelock
        assert_ok!(Fusion::create_escrow(
            RuntimeOrigin::signed(1),
            2u64,
            AssetInfo::Native,
            1000u128,
            b"test_secret_hash".to_vec(),
            10u64, // Short timelock
            b"test metadata".to_vec(),
            None,
        ));
        
        assert_ok!(Fusion::fund_escrow(RuntimeOrigin::signed(1), 1u64));
        
        // Wait for timelock to expire
        run_to_block(20);
        
        // Cancel the escrow
        assert_ok!(Fusion::cancel_escrow(
            RuntimeOrigin::signed(1),
            1u64,
            b"Timelock expired".to_vec(),
        ));
        
        // Check that the escrow state was updated
        let escrow = Fusion::get_escrow(&1u64).unwrap();
        assert_eq!(escrow.state, EscrowState::Cancelled);
    });
}

#[test]
fn emergency_pause_works() {
    new_test_ext().execute_with(|| {
        // Pause the pallet
        assert_ok!(Fusion::emergency_pause(RuntimeOrigin::signed(1)));
        
        // Try to create escrow - should fail
        assert_noop!(
            Fusion::create_escrow(
                RuntimeOrigin::signed(1),
                2u64,
                AssetInfo::Native,
                1000u128,
                b"test_secret_hash".to_vec(),
                100u64,
                b"test metadata".to_vec(),
                None,
            ),
            Error::<Test>::EmergencyPaused
        );
        
        // Unpause the pallet
        assert_ok!(Fusion::emergency_unpause(RuntimeOrigin::signed(1)));
        
        // Now creation should work
        assert_ok!(Fusion::create_escrow(
            RuntimeOrigin::signed(1),
            2u64,
            AssetInfo::Native,
            1000u128,
            b"test_secret_hash".to_vec(),
            100u64,
            b"test metadata".to_vec(),
            None,
        ));
    });
}

#[test]
fn invalid_secret_fails() {
    new_test_ext().execute_with(|| {
        let secret = b"test_secret";
        let hashlock = sp_core::hashing::sha2_256(secret).to_vec();
        
        // Create and fund escrow
        assert_ok!(Fusion::create_escrow(
            RuntimeOrigin::signed(1),
            2u64,
            AssetInfo::Native,
            1000u128,
            hashlock,
            100u64,
            b"test metadata".to_vec(),
            None,
        ));
        
        assert_ok!(Fusion::fund_escrow(RuntimeOrigin::signed(1), 1u64));
        
        // Try to complete with wrong secret
        assert_noop!(
            Fusion::complete_escrow(
                RuntimeOrigin::signed(2),
                1u64,
                b"wrong_secret".to_vec(),
            ),
            Error::<Test>::IncorrectSecret
        );
    });
}
