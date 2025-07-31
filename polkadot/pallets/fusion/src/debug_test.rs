#[test]
fn debug_create_escrow() {
    new_test_ext().execute_with(|| {
        println!("Starting test...");
        
        let beneficiary = 2u64;
        let asset = AssetInfo::Native;
        let amount = 1000u128;
        let hashlock = b"test_secret_hash".to_vec();
        let timelock_duration = 100u64;
        let metadata = b"test metadata".to_vec();
        let xcm_route = None;
        
        println!("About to call create_escrow...");
        
        let result = Fusion::create_escrow(
            RuntimeOrigin::signed(1),
            beneficiary,
            asset,
            amount,
            hashlock,
            timelock_duration,
            metadata,
            xcm_route,
        );
        
        println!("Create escrow result: {:?}", result);
        
        if result.is_ok() {
            println!("Next escrow ID: {:?}", Fusion::next_escrow_id());
            let escrow_opt = Fusion::get_escrow(&1u64);
            println!("Escrow lookup: {:?}", escrow_opt);
        }
    });
}
