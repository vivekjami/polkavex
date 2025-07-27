// SPDX-License-Identifier: MIT

pragma solidity ^0.8.28;


import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

/**
 * @title PolkavexEscrow
 * @dev Basic escrow contract for Ethereum side of cross-chain swaps
 * Implements hashlock/timelock mechanism compatible with 1inch Fusion+
 * Day 0 version - will be enhanced with full Fusion+ integration in Day 1
 */
contract PolkavexEscrow is ReentrancyGuard {
    struct Escrow {
        bytes32 secretHash;
        uint256 timelock;
        address maker;
        address resolver;
        IERC20 token;
        uint256 amount;
        bool completed;
        bool cancelled;
    }

    mapping(bytes32 => Escrow) public escrows;
    
    event EscrowCreated(
        bytes32 indexed secretHash,
        address indexed maker,
        address indexed resolver,
        address token,
        uint256 amount,
        uint256 timelock
    );
    
    event EscrowResolved(
        bytes32 indexed secretHash,
        bytes32 secret,
        address indexed resolver
    );
    
    event EscrowCancelled(bytes32 indexed secretHash);

    modifier onlyValidEscrow(bytes32 secretHash) {
        require(escrows[secretHash].amount > 0, "Escrow does not exist");
        require(!escrows[secretHash].completed, "Escrow already completed");
        require(!escrows[secretHash].cancelled, "Escrow already cancelled");
        _;
    }

    /**
     * @dev Create new escrow with hashlock and timelock
     */
    function createEscrow(
        bytes32 _secretHash,
        uint256 _timelock,
        address _resolver,
        IERC20 _token,
        uint256 _amount
    ) external {
        require(_timelock > block.timestamp, "Timelock must be in future");
        require(_amount > 0, "Amount must be greater than zero");
        require(escrows[_secretHash].amount == 0, "Escrow already exists");

        // Transfer tokens to this contract
        require(_token.transferFrom(msg.sender, address(this), _amount), "Transfer failed");

        escrows[_secretHash] = Escrow({
            secretHash: _secretHash,
            timelock: _timelock,
            maker: msg.sender,
            resolver: _resolver,
            token: _token,
            amount: _amount,
            completed: false,
            cancelled: false
        });

        emit EscrowCreated(_secretHash, msg.sender, _resolver, address(_token), _amount, _timelock);
    }

    /**
     * @dev Resolve escrow with secret (reveal phase)
     */
    function resolveEscrow(
        bytes32 _secret
    ) external nonReentrant onlyValidEscrow(keccak256(abi.encodePacked(_secret))) {
        bytes32 secretHash = keccak256(abi.encodePacked(_secret));
        Escrow storage escrow = escrows[secretHash];

        require(msg.sender == escrow.resolver, "Only resolver can resolve");

        escrow.completed = true;
        
        // Transfer tokens to resolver
        require(escrow.token.transfer(escrow.resolver, escrow.amount), "Transfer failed");

        emit EscrowResolved(secretHash, _secret, escrow.resolver);
    }

    /**
     * @dev Cancel escrow after timelock expires
     */
    function cancelEscrow(
        bytes32 _secretHash
    ) external nonReentrant onlyValidEscrow(_secretHash) {
        Escrow storage escrow = escrows[_secretHash];
        
        require(block.timestamp >= escrow.timelock, "Timelock not expired");
        require(msg.sender == escrow.maker, "Only maker can cancel");

        escrow.cancelled = true;

        // Return tokens to maker
        require(escrow.token.transfer(escrow.maker, escrow.amount), "Transfer failed");

        emit EscrowCancelled(_secretHash);
    }

    /**
     * @dev Get escrow details
     */
    function getEscrow(bytes32 _secretHash) external view returns (Escrow memory) {
        return escrows[_secretHash];
    }
}
