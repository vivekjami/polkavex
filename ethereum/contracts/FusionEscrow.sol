// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC721/IERC721.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

/**
 * @title FusionEscrow
 * @dev Enhanced escrow contract for 1inch Fusion+ cross-chain swaps between Ethereum and Polkadot.
 * Supports ETH, ERC-20 tokens, and ERC-721 NFTs with hashlock and timelock security.
 * 
 * Day 1 Implementation Features:
 * - Multi-asset support (ETH, ERC-20, ERC-721)
 * - Hashlock/Timelock atomic swap mechanism  
 * - 1inch Fusion+ compatible order structure
 * - Comprehensive event logging for UI tracking
 * - Gas-optimized storage and operations
 * - Security hardened with ReentrancyGuard and comprehensive checks
 */
contract FusionEscrow is Ownable, ReentrancyGuard {
    
    // Constructor to initialize Ownable with the deployer as owner
    constructor() Ownable(msg.sender) {}
    
    // Asset types supported by the escrow
    enum AssetType { ETH, ERC20, ERC721 }
    
    // Escrow state tracking
    enum EscrowState { CREATED, DEPOSITED, COMPLETED, CANCELLED }
    
    // Struct to store escrow details - optimized for gas efficiency
    struct Escrow {
        bytes32 secretHash;        // Hash of the secret for hashlock
        uint256 timelock;          // Block timestamp after which funds can be refunded
        address maker;             // User initiating the swap (Ethereum side)
        address taker;             // Expected claimer (Polkadot resolver)
        address assetAddress;      // Address of asset (address(0) for ETH, else token contract)
        uint256 amountOrId;        // Amount for ETH/ERC-20 or tokenId for ERC-721
        AssetType assetType;       // Type of asset being escrowed
        EscrowState state;         // Current state of the escrow
        uint256 createdAt;         // Block timestamp of creation
    }

    // Storage
    mapping(uint256 => Escrow) public escrows;
    mapping(bytes32 => uint256) public secretHashToEscrowId; // For quick lookup
    uint256 public escrowCounter;
    
    // Constants for validation
    uint256 public constant MIN_TIMELOCK_DURATION = 1 hours;
    uint256 public constant MAX_TIMELOCK_DURATION = 30 days;
    
    // Events for comprehensive on-chain tracking
    event EscrowCreated(
        uint256 indexed escrowId, 
        bytes32 indexed secretHash,
        address indexed maker, 
        address taker,
        address assetAddress,
        uint256 amountOrId,
        AssetType assetType,
        uint256 timelock
    );
    
    event EscrowDeposited(
        uint256 indexed escrowId, 
        address indexed maker,
        address assetAddress, 
        uint256 amountOrId
    );
    
    event EscrowCompleted(
        uint256 indexed escrowId, 
        bytes32 indexed secretHash,
        bytes32 secret,
        address indexed taker
    );
    
    event EscrowCancelled(
        uint256 indexed escrowId, 
        address indexed maker,
        string reason
    );

    // Custom errors for gas efficiency
    error InvalidTimelock();
    error EscrowNotFound();
    error UnauthorizedAccess();
    error InvalidEscrowState();
    error InvalidSecret();
    error TimelockNotExpired();
    error InsufficientValue();
    error TransferFailed();
    error SecretHashAlreadyUsed();

    /**
     * @dev Create a new escrow with hashlock and timelock for 1inch Fusion+ compatibility
     * @param secretHash Hash of the secret (keccak256(secret))
     * @param timelock Timestamp (in seconds) after which escrow can be cancelled
     * @param taker Expected claimer address (Polkadot resolver)
     * @param assetAddress Address of asset (address(0) for ETH, token address for ERC-20/ERC-721)
     * @param amountOrId Amount for fungible tokens or tokenId for NFTs
     * @param assetType Type of asset (ETH, ERC20, or ERC721)
     * @return escrowId Unique identifier for the created escrow
     */
    function createEscrow(
        bytes32 secretHash,
        uint256 timelock,
        address taker,
        address assetAddress,
        uint256 amountOrId,
        AssetType assetType
    ) external returns (uint256 escrowId) {
        // Validation
        if (timelock <= block.timestamp + MIN_TIMELOCK_DURATION || 
            timelock > block.timestamp + MAX_TIMELOCK_DURATION) {
            revert InvalidTimelock();
        }
        
        if (secretHashToEscrowId[secretHash] != 0) {
            revert SecretHashAlreadyUsed();
        }
        
        if (taker == address(0) || taker == msg.sender) {
            revert UnauthorizedAccess();
        }

        // Create escrow
        escrowId = ++escrowCounter;
        escrows[escrowId] = Escrow({
            secretHash: secretHash,
            timelock: timelock,
            maker: msg.sender,
            taker: taker,
            assetAddress: assetAddress,
            amountOrId: amountOrId,
            assetType: assetType,
            state: EscrowState.CREATED,
            createdAt: block.timestamp
        });
        
        secretHashToEscrowId[secretHash] = escrowId;
        
        emit EscrowCreated(
            escrowId, 
            secretHash, 
            msg.sender, 
            taker,
            assetAddress, 
            amountOrId, 
            assetType, 
            timelock
        );
        
        return escrowId;
    }

    /**
     * @dev Deposit assets into escrow (ETH, ERC-20, or ERC-721)
     * @param escrowId ID of the escrow to deposit into
     */
    function deposit(uint256 escrowId) external payable nonReentrant {
        Escrow storage escrow = escrows[escrowId];
        
        if (escrow.maker == address(0)) revert EscrowNotFound();
        if (msg.sender != escrow.maker) revert UnauthorizedAccess();
        if (escrow.state != EscrowState.CREATED) revert InvalidEscrowState();

        if (escrow.assetType == AssetType.ETH) {
            // ETH deposit
            if (msg.value != escrow.amountOrId) revert InsufficientValue();
        } else if (escrow.assetType == AssetType.ERC721) {
            // NFT deposit (ERC-721)
            IERC721(escrow.assetAddress).transferFrom(
                msg.sender, 
                address(this), 
                escrow.amountOrId
            );
        } else {
            // ERC-20 deposit
            IERC20(escrow.assetAddress).transferFrom(
                msg.sender, 
                address(this), 
                escrow.amountOrId
            );
        }
        
        escrow.state = EscrowState.DEPOSITED;
        
        emit EscrowDeposited(
            escrowId, 
            msg.sender, 
            escrow.assetAddress, 
            escrow.amountOrId
        );
    }

    /**
     * @dev Complete escrow by providing the correct secret (Fusion+ execution phase)
     * @param escrowId ID of the escrow
     * @param secret The secret whose hash matches secretHash
     */
    function complete(uint256 escrowId, bytes32 secret) external nonReentrant {
        Escrow storage escrow = escrows[escrowId];
        
        if (escrow.maker == address(0)) revert EscrowNotFound();
        if (escrow.state != EscrowState.DEPOSITED) revert InvalidEscrowState();
        if (keccak256(abi.encodePacked(secret)) != escrow.secretHash) revert InvalidSecret();
        
        // Anyone can complete with the correct secret, but funds go to designated taker
        escrow.state = EscrowState.COMPLETED;
        
        // Transfer assets to taker
        if (escrow.assetType == AssetType.ETH) {
            (bool success, ) = escrow.taker.call{value: escrow.amountOrId}("");
            if (!success) revert TransferFailed();
        } else if (escrow.assetType == AssetType.ERC721) {
            IERC721(escrow.assetAddress).transferFrom(
                address(this), 
                escrow.taker, 
                escrow.amountOrId
            );
        } else {
            bool success = IERC20(escrow.assetAddress).transfer(
                escrow.taker, 
                escrow.amountOrId
            );
            if (!success) revert TransferFailed();
        }
        
        emit EscrowCompleted(escrowId, escrow.secretHash, secret, escrow.taker);
    }

    /**
     * @dev Cancel escrow after timelock expires and refund to maker
     * @param escrowId ID of the escrow to cancel
     */
    function cancel(uint256 escrowId) external nonReentrant {
        Escrow storage escrow = escrows[escrowId];
        
        if (escrow.maker == address(0)) revert EscrowNotFound();
        if (msg.sender != escrow.maker) revert UnauthorizedAccess();
        if (escrow.state != EscrowState.DEPOSITED) revert InvalidEscrowState();
        if (block.timestamp < escrow.timelock) revert TimelockNotExpired();
        
        escrow.state = EscrowState.CANCELLED;
        
        // Refund assets to maker
        if (escrow.assetType == AssetType.ETH) {
            (bool success, ) = escrow.maker.call{value: escrow.amountOrId}("");
            if (!success) revert TransferFailed();
        } else if (escrow.assetType == AssetType.ERC721) {
            IERC721(escrow.assetAddress).transferFrom(
                address(this), 
                escrow.maker, 
                escrow.amountOrId
            );
        } else {
            bool success = IERC20(escrow.assetAddress).transfer(
                escrow.maker, 
                escrow.amountOrId
            );
            if (!success) revert TransferFailed();
        }
        
        emit EscrowCancelled(escrowId, escrow.maker, "Timelock expired");
    }

    /**
     * @dev Emergency cancel by maker before deposit (if needed)
     * @param escrowId ID of the escrow to cancel
     */
    function cancelBeforeDeposit(uint256 escrowId) external {
        Escrow storage escrow = escrows[escrowId];
        
        if (escrow.maker == address(0)) revert EscrowNotFound();
        if (msg.sender != escrow.maker) revert UnauthorizedAccess();
        if (escrow.state != EscrowState.CREATED) revert InvalidEscrowState();
        
        escrow.state = EscrowState.CANCELLED;
        
        emit EscrowCancelled(escrowId, escrow.maker, "Cancelled before deposit");
    }

    // View functions for UI and integration
    
    /**
     * @dev Get detailed escrow information
     * @param escrowId ID of the escrow
     * @return escrow Full escrow details
     */
    function getEscrow(uint256 escrowId) external view returns (Escrow memory escrow) {
        return escrows[escrowId];
    }
    
    /**
     * @dev Get escrow ID by secret hash
     * @param secretHash The secret hash to lookup
     * @return escrowId The corresponding escrow ID (0 if not found)
     */
    function getEscrowIdBySecretHash(bytes32 secretHash) external view returns (uint256 escrowId) {
        return secretHashToEscrowId[secretHash];
    }
    
    /**
     * @dev Check if escrow is active and deposited
     * @param escrowId ID of the escrow
     * @return isActive True if escrow can be completed
     */
    function isEscrowActive(uint256 escrowId) external view returns (bool isActive) {
        return escrows[escrowId].state == EscrowState.DEPOSITED;
    }
    
    /**
     * @dev Get time remaining until timelock expires
     * @param escrowId ID of the escrow
     * @return timeRemaining Seconds until timelock (0 if expired)
     */
    function getTimeRemaining(uint256 escrowId) external view returns (uint256 timeRemaining) {
        uint256 timelock = escrows[escrowId].timelock;
        return timelock > block.timestamp ? timelock - block.timestamp : 0;
    }

    // Admin functions for potential upgrades (Day 2+)
    
    /**
     * @dev Emergency pause function (for critical bugs only)
     * @dev Only owner can call - use carefully in hackathon context
     */
    function emergencyPause() external onlyOwner {
        // Implementation for emergency scenarios
        // This is a placeholder for potential Day 2+ enhancements
    }

    // Required for receiving NFTs
    function onERC721Received(
        address,
        address,
        uint256,
        bytes calldata
    ) external pure returns (bytes4) {
        return this.onERC721Received.selector;
    }
}
