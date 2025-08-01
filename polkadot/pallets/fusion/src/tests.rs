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
        
        // Check that the escrow was created with ID 0
        let escrow = Fusion::get_escrow(&0u64).unwrap();
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
        
        // Then fund it (escrow ID is 0)
        assert_ok!(Fusion::fund_escrow(RuntimeOrigin::signed(1), 0u64));
        
        // Check that the escrow state was updated
        let escrow = Fusion::get_escrow(&0u64).unwrap();
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
        
        assert_ok!(Fusion::fund_escrow(RuntimeOrigin::signed(1), 0u64));
        
        // Complete the escrow with the secret
        assert_ok!(Fusion::complete_escrow(
            RuntimeOrigin::signed(2),
            0u64,
            secret.to_vec(),
        ));
        
        // Check that the escrow state was updated
        let escrow = Fusion::get_escrow(&0u64).unwrap();
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
        
        assert_ok!(Fusion::fund_escrow(RuntimeOrigin::signed(1), 0u64));
        
        // Wait for timelock to expire
        run_to_block(20);
        
        // Cancel the escrow
        assert_ok!(Fusion::cancel_escrow(
            RuntimeOrigin::signed(1),
            0u64,
            b"Timelock expired".to_vec(),
        ));
        
        // Check that the escrow state was updated
        let escrow = Fusion::get_escrow(&0u64).unwrap();
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
        
        assert_ok!(Fusion::fund_escrow(RuntimeOrigin::signed(1), 0u64));
        
        // Try to complete with wrong secret
        assert_noop!(
            Fusion::complete_escrow(
                RuntimeOrigin::signed(2),
                0u64,
                b"wrong_secret".to_vec(),
            ),
            Error::<Test>::IncorrectSecret
        );
    });
}

// ===== Day 5 Enhancement Tests: NFT and Stablecoin Support =====

#[test]
fn create_stablecoin_escrow_works() {
    new_test_ext().execute_with(|| {
        let beneficiary = 2u64;
        let stablecoin_asset = AssetInfo::Stablecoin { 
            asset_id: 1u32, 
            decimals: 6u8, 
            symbol: b"USDC".to_vec().try_into().unwrap() 
        };
        let amount = 1000_000_000u128; // 1000 USDC with 6 decimals
        let hashlock = b"test_secret_hash".to_vec();
        let timelock_duration = 100u64;
        let metadata = b"USDC cross-chain swap".to_vec();
        
        assert_ok!(Fusion::create_escrow(
            RuntimeOrigin::signed(1),
            beneficiary,
            stablecoin_asset.clone(),
            amount,
            hashlock,
            timelock_duration,
            metadata,
            None,
        ));
        
        let escrow = Fusion::get_escrow(&0u64).unwrap();
        assert_eq!(escrow.beneficiary, beneficiary);
        assert_eq!(escrow.amount, amount);
        assert_eq!(escrow.asset, stablecoin_asset);
        assert_eq!(escrow.state, EscrowState::Created);
    });
}

#[test]
fn create_nft_escrow_works() {
    new_test_ext().execute_with(|| {
        let beneficiary = 2u64;
        let nft_asset = AssetInfo::Nft { 
            collection_id: 1u32, 
            item_id: 42u32, 
            metadata: b"CryptoPunk #42".to_vec().try_into().unwrap() 
        };
        let amount = 1u128; // NFTs have amount = 1
        let hashlock = b"nft_secret_hash".to_vec();
        let timelock_duration = 200u64;
        let metadata = b"NFT cross-chain transfer".to_vec();
        
        assert_ok!(Fusion::create_escrow(
            RuntimeOrigin::signed(1),
            beneficiary,
            nft_asset.clone(),
            amount,
            hashlock,
            timelock_duration,
            metadata,
            None,
        ));
        
        let escrow = Fusion::get_escrow(&0u64).unwrap();
        assert_eq!(escrow.beneficiary, beneficiary);
        assert_eq!(escrow.amount, amount);
        assert_eq!(escrow.asset, nft_asset);
        assert_eq!(escrow.state, EscrowState::Created);
    });
}

#[test]
fn asset_type_detection_works() {
    new_test_ext().execute_with(|| {
        // Test native asset type
        let native_asset: AssetInfo<u32> = AssetInfo::Native;
        assert_eq!(native_asset.asset_type(), crate::AssetType::Native);
        
        // Test fungible asset type
        let fungible_asset: AssetInfo<u32> = AssetInfo::Asset(1u32);
        assert_eq!(fungible_asset.asset_type(), crate::AssetType::Fungible);
        
        // Test stablecoin asset type
        let stablecoin_asset: AssetInfo<u32> = AssetInfo::Stablecoin { 
            asset_id: 1u32, 
            decimals: 6u8, 
            symbol: b"USDC".to_vec().try_into().unwrap() 
        };
        assert_eq!(stablecoin_asset.asset_type(), crate::AssetType::Stablecoin);
        
        // Test NFT asset type
        let nft_asset: AssetInfo<u32> = AssetInfo::Nft { 
            collection_id: 1u32, 
            item_id: 42u32, 
            metadata: b"Test NFT".to_vec().try_into().unwrap() 
        };
        assert_eq!(nft_asset.asset_type(), crate::AssetType::Nft);
    });
}

#[test]
fn stablecoin_fund_and_complete_works() {
    new_test_ext().execute_with(|| {
        let beneficiary = 2u64;
        let stablecoin_asset = AssetInfo::Stablecoin { 
            asset_id: 1u32, 
            decimals: 6u8, 
            symbol: b"USDC".to_vec().try_into().unwrap() 
        };
        let amount = 1000_000_000u128;
        let secret = b"secret123".to_vec();
        let hashlock = sp_core::blake2_256(&secret).to_vec();
        
        // Create escrow
        assert_ok!(Fusion::create_escrow(
            RuntimeOrigin::signed(1),
            beneficiary,
            stablecoin_asset,
            amount,
            hashlock,
            100u64,
            b"USDC swap".to_vec(),
            None,
        ));
        
        // Fund escrow (this would normally transfer assets)
        assert_ok!(Fusion::fund_escrow(RuntimeOrigin::signed(1), 0u64));
        
        let escrow = Fusion::get_escrow(&0u64).unwrap();
        assert_eq!(escrow.state, EscrowState::Active);
        
        // Complete escrow
        assert_ok!(Fusion::complete_escrow(
            RuntimeOrigin::signed(beneficiary),
            0u64,
            secret,
        ));
        
        let escrow = Fusion::get_escrow(&0u64).unwrap();
        assert_eq!(escrow.state, EscrowState::Completed);
    });
}

#[test]
fn nft_fund_and_complete_works() {
    new_test_ext().execute_with(|| {
        let beneficiary = 3u64;
        let nft_asset = AssetInfo::Nft { 
            collection_id: 1u32, 
            item_id: 123u32, 
            metadata: b"Rare NFT #123".to_vec().try_into().unwrap() 
        };
        let amount = 1u128;
        let secret = b"nft_secret_456".to_vec();
        let hashlock = sp_core::blake2_256(&secret).to_vec();
        
        // Create NFT escrow
        assert_ok!(Fusion::create_escrow(
            RuntimeOrigin::signed(1),
            beneficiary,
            nft_asset,
            amount,
            hashlock,
            150u64,
            b"NFT transfer".to_vec(),
            None,
        ));
        
        // Fund escrow (this would normally transfer the NFT)
        assert_ok!(Fusion::fund_escrow(RuntimeOrigin::signed(1), 0u64));
        
        let escrow = Fusion::get_escrow(&0u64).unwrap();
        assert_eq!(escrow.state, EscrowState::Active);
        
        // Complete escrow
        assert_ok!(Fusion::complete_escrow(
            RuntimeOrigin::signed(beneficiary),
            0u64,
            secret,
        ));
        
        let escrow = Fusion::get_escrow(&0u64).unwrap();
        assert_eq!(escrow.state, EscrowState::Completed);
    });
}

#[test]
fn mixed_asset_escrows_work() {
    new_test_ext().execute_with(|| {
        // Create multiple escrows with different asset types
        
        // Native DOT escrow
        assert_ok!(Fusion::create_escrow(
            RuntimeOrigin::signed(1),
            2u64,
            AssetInfo::Native,
            1000u128,
            b"native_hash".to_vec(),
            100u64,
            b"DOT swap".to_vec(),
            None,
        ));
        
        // Stablecoin escrow
        assert_ok!(Fusion::create_escrow(
            RuntimeOrigin::signed(1),
            2u64,
            AssetInfo::Stablecoin { 
                asset_id: 1u32, 
                decimals: 6u8, 
                symbol: b"USDT".to_vec().try_into().unwrap() 
            },
            2000_000_000u128,
            b"stable_hash".to_vec(),
            200u64,
            b"USDT swap".to_vec(),
            None,
        ));
        
        // NFT escrow
        assert_ok!(Fusion::create_escrow(
            RuntimeOrigin::signed(1),
            2u64,
            AssetInfo::Nft { 
                collection_id: 2u32, 
                item_id: 999u32, 
                metadata: b"Limited Edition #999".to_vec().try_into().unwrap() 
            },
            1u128,
            b"nft_hash".to_vec(),
            300u64,
            b"NFT transfer".to_vec(),
            None,
        ));
        
        // Check all escrows were created correctly
        assert_eq!(Fusion::get_escrow(&0u64).unwrap().asset.asset_type(), crate::AssetType::Native);
        assert_eq!(Fusion::get_escrow(&1u64).unwrap().asset.asset_type(), crate::AssetType::Stablecoin);
        assert_eq!(Fusion::get_escrow(&2u64).unwrap().asset.asset_type(), crate::AssetType::Nft);
        
        // Verify next escrow ID incremented correctly
        assert_eq!(Fusion::next_escrow_id(), 3u64);
    });
}

#[test]
fn enhanced_asset_security_validations() {
    new_test_ext().execute_with(|| {
        // Test edge cases for asset validation
        
        // Test stablecoin with maximum symbol length
        let max_symbol = vec![b'A'; 32]; // Maximum 32 bytes
        let stablecoin_max = AssetInfo::Stablecoin { 
            asset_id: 1u32, 
            decimals: 18u8, 
            symbol: max_symbol.try_into().unwrap() 
        };
        
        assert_ok!(Fusion::create_escrow(
            RuntimeOrigin::signed(1),
            2u64,
            stablecoin_max,
            1000u128,
            b"max_symbol_hash".to_vec(),
            100u64,
            b"Max symbol test".to_vec(),
            None,
        ));
        
        // Test NFT with maximum metadata length
        let max_metadata = vec![b'M'; 256]; // Maximum 256 bytes
        let nft_max = AssetInfo::Nft { 
            collection_id: 1u32, 
            item_id: 1u32, 
            metadata: max_metadata.try_into().unwrap() 
        };
        
        assert_ok!(Fusion::create_escrow(
            RuntimeOrigin::signed(1),
            2u64,
            nft_max,
            1u128,
            b"max_meta_hash".to_vec(),
            100u64,
            b"Max metadata test".to_vec(),
            None,
        ));
        
        // Verify both escrows were created successfully
        assert!(Fusion::get_escrow(&0u64).is_some());
        assert!(Fusion::get_escrow(&1u64).is_some());
    });
}
