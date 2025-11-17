#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Address, Vec, symbol_short, Symbol};

// Struct to store custody account details
#[contracttype]
#[derive(Clone)]
pub struct CustodyAccount {
    pub owner: Address,
    pub balance: i128,
    pub required_signatures: u32,
    pub is_insured: bool,
    pub is_active: bool,
}

// Mapping for custody accounts
#[contracttype]
pub enum CustodyBook {
    Account(Address)
}

// Counter for total custody accounts
const TOTAL_ACCOUNTS: Symbol = symbol_short!("TOT_ACC");

#[contract]
pub struct AssetCustodyContract;

#[contractimpl]
impl AssetCustodyContract {
    
    // Function to create a new custody account with multi-signature protection
    pub fn create_custody_account(
        env: Env, 
        owner: Address, 
        required_signatures: u32,
        insurance: bool
    ) -> bool {
        owner.require_auth();
        
        let account_key = CustodyBook::Account(owner.clone());
        
        // Check if account already exists
        let existing: Option<CustodyAccount> = env.storage().instance().get(&account_key);
        
        if existing.is_some() {
            log!(&env, "Custody account already exists for this address");
            return false;
        }
        
        // Validate required signatures (minimum 2 for multi-sig)
        if required_signatures < 2 {
            log!(&env, "Multi-signature requires at least 2 signatures");
            panic!("Minimum 2 signatures required");
        }
        
        // Create new custody account
        let new_account = CustodyAccount {
            owner: owner.clone(),
            balance: 0,
            required_signatures,
            is_insured: insurance,
            is_active: true,
        };
        
        // Store the account
        env.storage().instance().set(&account_key, &new_account);
        
        // Update total accounts counter
        let mut total: u64 = env.storage().instance().get(&TOTAL_ACCOUNTS).unwrap_or(0);
        total += 1;
        env.storage().instance().set(&TOTAL_ACCOUNTS, &total);
        
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Custody account created successfully for owner");
        true
    }
    
    // Function to deposit assets into custody
    pub fn deposit_assets(env: Env, owner: Address, amount: i128) -> bool {
        owner.require_auth();
        
        if amount <= 0 {
            log!(&env, "Deposit amount must be positive");
            return false;
        }
        
        let account_key = CustodyBook::Account(owner.clone());
        let mut account: CustodyAccount = env.storage().instance()
            .get(&account_key)
            .unwrap_or_else(|| panic!("Custody account not found"));
        
        if !account.is_active {
            log!(&env, "Custody account is not active");
            return false;
        }
        
        // Update balance
        account.balance += amount;
        
        // Store updated account
        env.storage().instance().set(&account_key, &account);
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Assets deposited successfully. New balance: {}", account.balance);
        true
    }
    
    // Function to withdraw assets with multi-signature verification
    pub fn withdraw_assets(
        env: Env, 
        owner: Address, 
        amount: i128,
        signatures_count: u32
    ) -> bool {
        owner.require_auth();
        
        if amount <= 0 {
            log!(&env, "Withdrawal amount must be positive");
            return false;
        }
        
        let account_key = CustodyBook::Account(owner.clone());
        let mut account: CustodyAccount = env.storage().instance()
            .get(&account_key)
            .unwrap_or_else(|| panic!("Custody account not found"));
        
        if !account.is_active {
            log!(&env, "Custody account is not active");
            return false;
        }
        
        // Verify multi-signature requirement
        if signatures_count < account.required_signatures {
            log!(&env, "Insufficient signatures for withdrawal. Required: {}, Provided: {}", 
                account.required_signatures, signatures_count);
            panic!("Multi-signature verification failed");
        }
        
        // Check sufficient balance
        if account.balance < amount {
            log!(&env, "Insufficient balance for withdrawal");
            return false;
        }
        
        // Update balance
        account.balance -= amount;
        
        // Store updated account
        env.storage().instance().set(&account_key, &account);
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Assets withdrawn successfully. Remaining balance: {}", account.balance);
        true
    }
    
    // Function to view custody account details
    pub fn view_custody_account(env: Env, owner: Address) -> CustodyAccount {
        let account_key = CustodyBook::Account(owner.clone());
        
        env.storage().instance().get(&account_key).unwrap_or(CustodyAccount {
            owner: owner.clone(),
            balance: 0,
            required_signatures: 0,
            is_insured: false,
            is_active: false,
        })
    }
}