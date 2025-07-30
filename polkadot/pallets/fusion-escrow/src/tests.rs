//! Comprehensive tests for the fusion-escrow pallet

use crate::{mock::*, Error, Event, AssetType, EscrowState};
use frame_support::{
    assert_err, assert_ok, assert_noop,
    traits::{Hooks, tokens::Preservation},
    BoundedVec,
};
use sp_core::blake2_256;
use sp_runtime::traits::BlakeTwo256;

#[test]
fn create_escrow_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let secret = b"test_secret_12345678901234567890";
        let secret_hash = blake2_256(secret);
        let timelock = 50;
        let amount = 1000;
        let metadata = BoundedVec::try_from(b"test metadata".to_vec()).unwrap();
        
        assert_ok!(FusionEscrow::create_escrow(
            RuntimeOrigin::signed(1),
            secret_hash,
            timelock,
            2, // taker
            AssetType::Native,
            amount,
            None, // no XCM destination
            metadata
        ));
        
        // Check that the escrow was created
        let escrow = FusionEscrow::get_escrow(1).unwrap();
        assert_eq!(escrow.secret_hash, secret_hash);
        assert_eq!(escrow.maker, 1);
        assert_eq!(escrow.taker, 2);
        assert_eq!(escrow.timelock, timelock);
        assert_eq!(escrow.amount, amount);
        assert_eq!(escrow.state, EscrowState::Created);
        
        // Check that the next escrow ID was incremented
        assert_eq!(FusionEscrow::next_escrow_id(), 2);
        
        // Check that the secret hash is indexed
        assert_eq!(FusionEscrow::get_escrow_by_secret(&secret_hash), Some(1));
        
        // Check that the event was emitted
        let events = System::events();
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0].event,
            RuntimeEvent::FusionEscrow(Event::EscrowCreated { escrow_id: 1, .. })
        ));
    });
}

#[test]
fn create_escrow_with_invalid_timelock_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let secret_hash = blake2_256(b"test_secret");
        let metadata = BoundedVec::try_from(b"test".to_vec()).unwrap();
        
        // Timelock too short
        assert_noop!(
            FusionEscrow::create_escrow(
                RuntimeOrigin::signed(1),
                secret_hash,
                5, // Less than MinTimelock (10)
                2,
                AssetType::Native,
                1000,
                None,
                metadata.clone()
            ),
            Error::<Test>::InvalidTimelock
        );
        
        // Timelock too long
        assert_noop!(
            FusionEscrow::create_escrow(
                RuntimeOrigin::signed(1),
                secret_hash,
                200000, // More than MaxTimelock (100800)
                2,
                AssetType::Native,
                1000,
                None,
                metadata
            ),
            Error::<Test>::InvalidTimelock
        );
    });
}

#[test]
fn create_escrow_with_same_secret_hash_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let secret_hash = blake2_256(b"test_secret");
        let metadata = BoundedVec::try_from(b"test".to_vec()).unwrap();
        
        // First escrow should succeed
        assert_ok!(FusionEscrow::create_escrow(
            RuntimeOrigin::signed(1),
            secret_hash,
            50,
            2,
            AssetType::Native,
            1000,
            None,
            metadata.clone()
        ));
        
        // Second escrow with same secret hash should fail
        assert_noop!(
            FusionEscrow::create_escrow(
                RuntimeOrigin::signed(1),
                secret_hash,
                50,
                3,
                AssetType::Native,
                1000,
                None,
                metadata
            ),
            Error::<Test>::DuplicateSecretHash
        );
    });
}

#[test]
fn create_escrow_with_same_maker_and_taker_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let secret_hash = blake2_256(b"test_secret");
        let metadata = BoundedVec::try_from(b"test".to_vec()).unwrap();
        
        assert_noop!(
            FusionEscrow::create_escrow(
                RuntimeOrigin::signed(1),
                secret_hash,
                50,
                1, // Same as maker
                AssetType::Native,
                1000,
                None,
                metadata
            ),
            Error::<Test>::InvalidTaker
        );
    });
}

#[test]
fn fund_escrow_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let secret_hash = blake2_256(b"test_secret");
        let amount = 1000;
        let metadata = BoundedVec::try_from(b"test".to_vec()).unwrap();
        
        // Create escrow
        assert_ok!(FusionEscrow::create_escrow(
            RuntimeOrigin::signed(1),
            secret_hash,
            50,
            2,
            AssetType::Native,
            amount,
            None,
            metadata
        ));
        
        let initial_balance = Balances::free_balance(1);
        let pallet_account = FusionEscrow::account_id();
        let initial_pallet_balance = Balances::free_balance(&pallet_account);
        
        // Fund the escrow
        assert_ok!(FusionEscrow::fund_escrow(RuntimeOrigin::signed(1), 1));
        
        // Check that the escrow state changed to Active
        let escrow = FusionEscrow::get_escrow(1).unwrap();
        assert_eq!(escrow.state, EscrowState::Active);
        
        // Check that the balance was transferred
        assert_eq!(Balances::free_balance(1), initial_balance - amount);
        assert_eq!(Balances::free_balance(&pallet_account), initial_pallet_balance + amount);
        
        // Check that the event was emitted
        let events = System::events();
        assert!(events.iter().any(|e| matches!(
            e.event,
            RuntimeEvent::FusionEscrow(Event::EscrowFunded { escrow_id: 1, .. })
        )));
    });
}

#[test]
fn fund_escrow_by_non_maker_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let secret_hash = blake2_256(b"test_secret");
        let metadata = BoundedVec::try_from(b"test".to_vec()).unwrap();
        
        // Create escrow with maker = 1
        assert_ok!(FusionEscrow::create_escrow(
            RuntimeOrigin::signed(1),
            secret_hash,
            50,
            2,
            AssetType::Native,
            1000,
            None,
            metadata
        ));
        
        // Try to fund by non-maker
        assert_noop!(
            FusionEscrow::fund_escrow(RuntimeOrigin::signed(2), 1),
            Error::<Test>::NotAuthorized
        );
    });
}

#[test]
fn fund_escrow_after_timelock_expires_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let secret_hash = blake2_256(b"test_secret");
        let timelock = 20;
        let metadata = BoundedVec::try_from(b"test".to_vec()).unwrap();
        
        // Create escrow
        assert_ok!(FusionEscrow::create_escrow(
            RuntimeOrigin::signed(1),
            secret_hash,
            timelock,
            2,
            AssetType::Native,
            1000,
            None,
            metadata
        ));
        
        // Move past timelock
        System::set_block_number(timelock + 1);
        
        // Try to fund after timelock
        assert_noop!(
            FusionEscrow::fund_escrow(RuntimeOrigin::signed(1), 1),
            Error::<Test>::TimelockExpired
        );
    });
}

#[test]
fn complete_escrow_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let secret = b"test_secret_12345678901234567890";
        let secret_hash = blake2_256(secret);
        let amount = 1000;
        let metadata = BoundedVec::try_from(b"test".to_vec()).unwrap();
        
        // Create and fund escrow
        assert_ok!(FusionEscrow::create_escrow(
            RuntimeOrigin::signed(1),
            secret_hash,
            50,
            2,
            AssetType::Native,
            amount,
            None,
            metadata
        ));
        
        assert_ok!(FusionEscrow::fund_escrow(RuntimeOrigin::signed(1), 1));
        
        let initial_taker_balance = Balances::free_balance(2);
        let pallet_account = FusionEscrow::account_id();
        let initial_pallet_balance = Balances::free_balance(&pallet_account);
        
        // Complete the escrow
        assert_ok!(FusionEscrow::complete_escrow(
            RuntimeOrigin::signed(2),
            1,
            *secret
        ));
        
        // Check that the escrow state changed to Completed
        let escrow = FusionEscrow::get_escrow(1).unwrap();
        assert_eq!(escrow.state, EscrowState::Completed);
        
        // Check that the balance was transferred to taker
        assert_eq!(Balances::free_balance(2), initial_taker_balance + amount);
        assert_eq!(Balances::free_balance(&pallet_account), initial_pallet_balance - amount);
        
        // Check that the event was emitted
        let events = System::events();
        assert!(events.iter().any(|e| matches!(
            e.event,
            RuntimeEvent::FusionEscrow(Event::EscrowCompleted { escrow_id: 1, .. })
        )));
    });
}

#[test]
fn complete_escrow_with_wrong_secret_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let secret = b"test_secret_12345678901234567890";
        let wrong_secret = b"wrong_secret123456789012345678901";
        let secret_hash = blake2_256(secret);
        let metadata = BoundedVec::try_from(b"test".to_vec()).unwrap();
        
        // Create and fund escrow
        assert_ok!(FusionEscrow::create_escrow(
            RuntimeOrigin::signed(1),
            secret_hash,
            50,
            2,
            AssetType::Native,
            1000,
            None,
            metadata
        ));
        
        assert_ok!(FusionEscrow::fund_escrow(RuntimeOrigin::signed(1), 1));
        
        // Try to complete with wrong secret
        assert_noop!(
            FusionEscrow::complete_escrow(RuntimeOrigin::signed(2), 1, *wrong_secret),
            Error::<Test>::InvalidSecret
        );
    });
}

#[test]
fn complete_escrow_after_timelock_expires_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let secret = b"test_secret_12345678901234567890";
        let secret_hash = blake2_256(secret);
        let timelock = 20;
        let metadata = BoundedVec::try_from(b"test".to_vec()).unwrap();
        
        // Create and fund escrow
        assert_ok!(FusionEscrow::create_escrow(
            RuntimeOrigin::signed(1),
            secret_hash,
            timelock,
            2,
            AssetType::Native,
            1000,
            None,
            metadata
        ));
        
        assert_ok!(FusionEscrow::fund_escrow(RuntimeOrigin::signed(1), 1));
        
        // Move past timelock
        System::set_block_number(timelock + 1);
        
        // Try to complete after timelock
        assert_noop!(
            FusionEscrow::complete_escrow(RuntimeOrigin::signed(2), 1, *secret),
            Error::<Test>::TimelockExpired
        );
    });
}

#[test]
fn cancel_escrow_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let secret_hash = blake2_256(b"test_secret");
        let timelock = 20;
        let amount = 1000;
        let metadata = BoundedVec::try_from(b"test".to_vec()).unwrap();
        
        // Create and fund escrow
        assert_ok!(FusionEscrow::create_escrow(
            RuntimeOrigin::signed(1),
            secret_hash,
            timelock,
            2,
            AssetType::Native,
            amount,
            None,
            metadata
        ));
        
        assert_ok!(FusionEscrow::fund_escrow(RuntimeOrigin::signed(1), 1));
        
        let initial_maker_balance = Balances::free_balance(1);
        let pallet_account = FusionEscrow::account_id();
        let initial_pallet_balance = Balances::free_balance(&pallet_account);
        
        // Move past timelock
        System::set_block_number(timelock + 1);
        
        // Cancel the escrow
        assert_ok!(FusionEscrow::cancel_escrow(RuntimeOrigin::signed(1), 1));
        
        // Check that the escrow state changed to Cancelled
        let escrow = FusionEscrow::get_escrow(1).unwrap();
        assert_eq!(escrow.state, EscrowState::Cancelled);
        
        // Check that the balance was refunded to maker
        assert_eq!(Balances::free_balance(1), initial_maker_balance + amount);
        assert_eq!(Balances::free_balance(&pallet_account), initial_pallet_balance - amount);
        
        // Check that the event was emitted
        let events = System::events();
        assert!(events.iter().any(|e| matches!(
            e.event,
            RuntimeEvent::FusionEscrow(Event::EscrowCancelled { escrow_id: 1, .. })
        )));
    });
}

#[test]
fn cancel_escrow_before_timelock_expires_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let secret_hash = blake2_256(b"test_secret");
        let timelock = 50;
        let metadata = BoundedVec::try_from(b"test".to_vec()).unwrap();
        
        // Create and fund escrow
        assert_ok!(FusionEscrow::create_escrow(
            RuntimeOrigin::signed(1),
            secret_hash,
            timelock,
            2,
            AssetType::Native,
            1000,
            None,
            metadata
        ));
        
        assert_ok!(FusionEscrow::fund_escrow(RuntimeOrigin::signed(1), 1));
        
        // Try to cancel before timelock expires
        assert_noop!(
            FusionEscrow::cancel_escrow(RuntimeOrigin::signed(1), 1),
            Error::<Test>::TimelockNotExpired
        );
    });
}

#[test]
fn cancel_escrow_by_non_maker_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let secret_hash = blake2_256(b"test_secret");
        let timelock = 20;
        let metadata = BoundedVec::try_from(b"test".to_vec()).unwrap();
        
        // Create and fund escrow
        assert_ok!(FusionEscrow::create_escrow(
            RuntimeOrigin::signed(1),
            secret_hash,
            timelock,
            2,
            AssetType::Native,
            1000,
            None,
            metadata
        ));
        
        assert_ok!(FusionEscrow::fund_escrow(RuntimeOrigin::signed(1), 1));
        
        // Move past timelock
        System::set_block_number(timelock + 1);
        
        // Try to cancel by non-maker
        assert_noop!(
            FusionEscrow::cancel_escrow(RuntimeOrigin::signed(2), 1),
            Error::<Test>::NotAuthorized
        );
    });
}

#[test]
fn cancel_before_funding_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let secret_hash = blake2_256(b"test_secret");
        let metadata = BoundedVec::try_from(b"test".to_vec()).unwrap();
        
        // Create escrow (but don't fund it)
        assert_ok!(FusionEscrow::create_escrow(
            RuntimeOrigin::signed(1),
            secret_hash,
            50,
            2,
            AssetType::Native,
            1000,
            None,
            metadata
        ));
        
        // Cancel before funding
        assert_ok!(FusionEscrow::cancel_before_funding(RuntimeOrigin::signed(1), 1));
        
        // Check that the escrow state changed to Cancelled
        let escrow = FusionEscrow::get_escrow(1).unwrap();
        assert_eq!(escrow.state, EscrowState::Cancelled);
        
        // Check that secret hash index was removed
        assert_eq!(FusionEscrow::get_escrow_by_secret(&secret_hash), None);
        
        // Check that the event was emitted
        let events = System::events();
        assert!(events.iter().any(|e| matches!(
            e.event,
            RuntimeEvent::FusionEscrow(Event::EscrowCancelled { escrow_id: 1, .. })
        )));
    });
}

#[test]
fn asset_escrow_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let secret = b"test_secret_12345678901234567890";
        let secret_hash = blake2_256(secret);
        let amount = 500;
        let asset_id = 0;
        let metadata = BoundedVec::try_from(b"test".to_vec()).unwrap();
        
        // Create asset escrow
        assert_ok!(FusionEscrow::create_escrow(
            RuntimeOrigin::signed(1),
            secret_hash,
            50,
            2,
            AssetType::Asset(asset_id),
            amount,
            None,
            metadata
        ));
        
        let initial_maker_balance = Assets::balance(asset_id, 1);
        let initial_taker_balance = Assets::balance(asset_id, 2);
        let pallet_account = FusionEscrow::account_id();
        
        // Fund the escrow
        assert_ok!(FusionEscrow::fund_escrow(RuntimeOrigin::signed(1), 1));
        
        // Check that assets were transferred to pallet
        assert_eq!(Assets::balance(asset_id, 1), initial_maker_balance - amount);
        assert_eq!(Assets::balance(asset_id, &pallet_account), amount);
        
        // Complete the escrow
        assert_ok!(FusionEscrow::complete_escrow(
            RuntimeOrigin::signed(2),
            1,
            *secret
        ));
        
        // Check that assets were transferred to taker
        assert_eq!(Assets::balance(asset_id, 2), initial_taker_balance + amount);
        assert_eq!(Assets::balance(asset_id, &pallet_account), 0);
        
        // Check escrow state
        let escrow = FusionEscrow::get_escrow(1).unwrap();
        assert_eq!(escrow.state, EscrowState::Completed);
    });
}

#[test]
fn emergency_pause_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        // Initially not paused
        assert_eq!(FusionEscrow::is_paused(), false);
        
        // Only root can pause
        assert_noop!(
            FusionEscrow::toggle_pause(RuntimeOrigin::signed(1)),
            sp_runtime::DispatchError::BadOrigin
        );
        
        // Pause the pallet
        assert_ok!(FusionEscrow::toggle_pause(RuntimeOrigin::root()));
        assert_eq!(FusionEscrow::is_paused(), true);
        
        // Check that the event was emitted
        let events = System::events();
        assert!(events.iter().any(|e| matches!(
            e.event,
            RuntimeEvent::FusionEscrow(Event::EmergencyPauseToggled { paused: true })
        )));
        
        // Try to create escrow while paused
        let secret_hash = blake2_256(b"test_secret");
        let metadata = BoundedVec::try_from(b"test".to_vec()).unwrap();
        
        assert_noop!(
            FusionEscrow::create_escrow(
                RuntimeOrigin::signed(1),
                secret_hash,
                50,
                2,
                AssetType::Native,
                1000,
                None,
                metadata
            ),
            Error::<Test>::PalletPaused
        );
        
        // Unpause the pallet
        assert_ok!(FusionEscrow::toggle_pause(RuntimeOrigin::root()));
        assert_eq!(FusionEscrow::is_paused(), false);
    });
}

#[test]
fn helper_functions_work() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let secret_hash = blake2_256(b"test_secret");
        let timelock = 50;
        let metadata = BoundedVec::try_from(b"test".to_vec()).unwrap();
        
        // Create and fund escrow
        assert_ok!(FusionEscrow::create_escrow(
            RuntimeOrigin::signed(1),
            secret_hash,
            timelock,
            2,
            AssetType::Native,
            1000,
            None,
            metadata
        ));
        
        assert_ok!(FusionEscrow::fund_escrow(RuntimeOrigin::signed(1), 1));
        
        // Test helper functions
        assert_eq!(FusionEscrow::is_escrow_active(1), true);
        assert_eq!(FusionEscrow::is_escrow_active(999), false);
        
        assert_eq!(FusionEscrow::get_time_remaining(1), Some(timelock - 1));
        assert_eq!(FusionEscrow::get_time_remaining(999), None);
        
        assert_eq!(FusionEscrow::get_escrows_by_maker(&1), vec![1]);
        assert_eq!(FusionEscrow::get_escrows_by_taker(&2), vec![1]);
        assert_eq!(FusionEscrow::get_escrows_by_maker(&999), vec![]);
    });
}
