#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, String, Symbol, Vec};

// ============================================================
// Data Types
// ============================================================

/// A payment split configuration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Split {
    pub split_id: u64,
    pub owner: Address,
    pub name: String,
    pub recipients: Vec<Address>,
    /// Shares in basis points — must sum to 10000 (100%)
    pub shares: Vec<u32>,
    pub active: bool,
}

/// Storage keys
#[contracttype]
pub enum DataKey {
    Split(u64),  // split_id -> Split
    Counter,     // total splits created
    Admin,       // contract admin
}

// ============================================================
// Contract
// ============================================================

#[contract]
pub struct SorobanSplitContract;

#[contractimpl]
impl SorobanSplitContract {
    // --------------------------------------------------------
    // Initialization
    // --------------------------------------------------------

    /// Initialize the contract with an admin address
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Counter, &0u64);

        env.events()
            .publish((Symbol::new(&env, "initialized"),), (admin,));
    }

    // --------------------------------------------------------
    // Split Management
    // --------------------------------------------------------

    /// Create a new payment split.
    /// Shares are in basis points — must sum to exactly 10000 (= 100%).
    /// Returns the new split ID.
    pub fn create_split(
        env: Env,
        owner: Address,
        name: String,
        recipients: Vec<Address>,
        shares: Vec<u32>,
    ) -> u64 {
        owner.require_auth();

        // Validate inputs
        assert!(!name.is_empty(), "Name cannot be empty");
        assert!(recipients.len() > 0, "Must have at least one recipient");
        assert!(
            recipients.len() == shares.len(),
            "Recipients and shares length mismatch"
        );
        assert!(
            Self::sum_shares(&shares) == 10000,
            "Shares must sum to 10000 basis points"
        );

        let counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::Counter)
            .unwrap_or(0);
        let split_id = counter + 1;

        let split = Split {
            split_id,
            owner: owner.clone(),
            name,
            recipients,
            shares,
            active: true,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Split(split_id), &split);
        env.storage()
            .instance()
            .set(&DataKey::Counter, &split_id);

        env.events()
            .publish((Symbol::new(&env, "split_created"),), (split_id, owner));

        split_id
    }

    /// Execute a split — transfers `amount` of `token` from `caller`
    /// to all recipients according to their shares.
    pub fn execute_split(
        env: Env,
        caller: Address,
        split_id: u64,
        token: Address,
        amount: i128,
    ) {
        caller.require_auth();
        assert!(amount > 0, "Amount must be greater than zero");

        let split: Split = env
            .storage()
            .persistent()
            .get(&DataKey::Split(split_id))
            .expect("Split not found");

        assert!(split.active, "Split is not active");

        let token_client = token::Client::new(&env, &token);

        for i in 0..split.recipients.len() {
            let recipient = split.recipients.get(i).unwrap();
            let share = split.shares.get(i).unwrap() as i128;
            let recipient_amount = amount * share / 10_000;
            if recipient_amount > 0 {
                token_client.transfer(&caller, &recipient, &recipient_amount);
            }
        }

        env.events().publish(
            (Symbol::new(&env, "split_executed"),),
            (split_id, caller, amount),
        );
    }

    /// Owner updates recipients and shares of an existing split.
    pub fn update_split(
        env: Env,
        owner: Address,
        split_id: u64,
        recipients: Vec<Address>,
        shares: Vec<u32>,
    ) {
        owner.require_auth();

        let mut split: Split = env
            .storage()
            .persistent()
            .get(&DataKey::Split(split_id))
            .expect("Split not found");

        assert!(split.owner == owner, "Not the split owner");
        assert!(split.active, "Split is not active");
        assert!(recipients.len() > 0, "Must have at least one recipient");
        assert!(
            recipients.len() == shares.len(),
            "Recipients and shares length mismatch"
        );
        assert!(
            Self::sum_shares(&shares) == 10000,
            "Shares must sum to 10000 basis points"
        );

        split.recipients = recipients;
        split.shares = shares;

        env.storage()
            .persistent()
            .set(&DataKey::Split(split_id), &split);

        env.events()
            .publish((Symbol::new(&env, "split_updated"),), (split_id, owner));
    }

    /// Owner deactivates a split — it can no longer be executed.
    pub fn deactivate_split(env: Env, owner: Address, split_id: u64) {
        owner.require_auth();

        let mut split: Split = env
            .storage()
            .persistent()
            .get(&DataKey::Split(split_id))
            .expect("Split not found");

        assert!(split.owner == owner, "Not the split owner");
        assert!(split.active, "Split already inactive");

        split.active = false;

        env.storage()
            .persistent()
            .set(&DataKey::Split(split_id), &split);

        env.events()
            .publish((Symbol::new(&env, "split_deactivated"),), (split_id, owner));
    }

    /// Owner reactivates a previously deactivated split.
    pub fn reactivate_split(env: Env, owner: Address, split_id: u64) {
        owner.require_auth();

        let mut split: Split = env
            .storage()
            .persistent()
            .get(&DataKey::Split(split_id))
            .expect("Split not found");

        assert!(split.owner == owner, "Not the split owner");
        assert!(!split.active, "Split is already active");

        split.active = true;

        env.storage()
            .persistent()
            .set(&DataKey::Split(split_id), &split);

        env.events()
            .publish((Symbol::new(&env, "split_reactivated"),), (split_id, owner));
    }

    // --------------------------------------------------------
    // Read / Query Functions
    // --------------------------------------------------------

    /// Get split details by ID
    pub fn get_split(env: Env, split_id: u64) -> Split {
        env.storage()
            .persistent()
            .get(&DataKey::Split(split_id))
            .expect("Split not found")
    }

    /// Get total number of splits created
    pub fn get_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::Counter)
            .unwrap_or(0)
    }

    /// Get recipients for a split
    pub fn get_recipients(env: Env, split_id: u64) -> Vec<Address> {
        let split: Split = env
            .storage()
            .persistent()
            .get(&DataKey::Split(split_id))
            .expect("Split not found");
        split.recipients
    }

    /// Get shares for a split
    pub fn get_shares(env: Env, split_id: u64) -> Vec<u32> {
        let split: Split = env
            .storage()
            .persistent()
            .get(&DataKey::Split(split_id))
            .expect("Split not found");
        split.shares
    }

    // --------------------------------------------------------
    // Internal Helpers
    // --------------------------------------------------------

    /// Sum all shares — must equal 10000 for a valid split
    fn sum_shares(shares: &Vec<u32>) -> u32 {
        let mut total: u32 = 0;
        for i in 0..shares.len() {
            total += shares.get(i).unwrap();
        }
        total
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::Address as _,
        token::StellarAssetClient,
        Address, Env, String, Vec,
    };

    fn setup(env: &Env) -> (Address, SorobanSplitContractClient) {
        env.mock_all_auths();
        let admin = Address::generate(env);
        let contract_id = env.register_contract(None, SorobanSplitContract);
        let client = SorobanSplitContractClient::new(env, &contract_id);
        client.initialize(&admin);
        (admin, client)
    }

    fn make_token(env: &Env) -> Address {
        let token_admin = Address::generate(env);
        env.register_stellar_asset_contract_v2(token_admin).address()
    }

    fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
        StellarAssetClient::new(env, token).mint(to, &amount);
    }

    fn make_split(
        env: &Env,
        client: &SorobanSplitContractClient,
        owner: &Address,
        recipients: Vec<Address>,
        shares: Vec<u32>,
    ) -> u64 {
        client.create_split(
            owner,
            &String::from_str(env, "Test Split"),
            &recipients,
            &shares,
        )
    }

    #[test]
    fn test_initialize() {
        let env = Env::default();
        let (_admin, client) = setup(&env);
        assert_eq!(client.get_count(), 0);
    }

    #[test]
    fn test_create_split() {
        let env = Env::default();
        let (_, client) = setup(&env);
        let owner = Address::generate(&env);
        let r1 = Address::generate(&env);
        let r2 = Address::generate(&env);

        let mut recipients = Vec::new(&env);
        recipients.push_back(r1.clone());
        recipients.push_back(r2.clone());

        let mut shares = Vec::new(&env);
        shares.push_back(7000u32);
        shares.push_back(3000u32);

        let id = client.create_split(
            &owner,
            &String::from_str(&env, "Revenue Split"),
            &recipients,
            &shares,
        );

        assert_eq!(id, 1);
        assert_eq!(client.get_count(), 1);

        let split = client.get_split(&id);
        assert_eq!(split.owner, owner);
        assert!(split.active);
        assert_eq!(split.shares.get(0).unwrap(), 7000u32);
        assert_eq!(split.shares.get(1).unwrap(), 3000u32);
    }

    #[test]
    fn test_execute_split_distributes_correctly() {
        let env = Env::default();
        let (_, client) = setup(&env);
        let token = make_token(&env);
        let owner = Address::generate(&env);
        let caller = Address::generate(&env);
        let r1 = Address::generate(&env);
        let r2 = Address::generate(&env);
        let r3 = Address::generate(&env);

        mint(&env, &token, &caller, 10_000);

        let mut recipients = Vec::new(&env);
        recipients.push_back(r1.clone());
        recipients.push_back(r2.clone());
        recipients.push_back(r3.clone());

        let mut shares = Vec::new(&env);
        shares.push_back(7000u32);
        shares.push_back(2000u32);
        shares.push_back(1000u32);

        let id = make_split(&env, &client, &owner, recipients, shares);
        client.execute_split(&caller, &id, &token, &10_000i128);

        let token_client = soroban_sdk::token::Client::new(&env, &token);
        assert_eq!(token_client.balance(&r1), 7000);
        assert_eq!(token_client.balance(&r2), 2000);
        assert_eq!(token_client.balance(&r3), 1000);
    }

    #[test]
    fn test_execute_single_recipient() {
        let env = Env::default();
        let (_, client) = setup(&env);
        let token = make_token(&env);
        let owner = Address::generate(&env);
        let caller = Address::generate(&env);
        let r1 = Address::generate(&env);

        mint(&env, &token, &caller, 5_000);

        let mut recipients = Vec::new(&env);
        recipients.push_back(r1.clone());

        let mut shares = Vec::new(&env);
        shares.push_back(10000u32);

        let id = make_split(&env, &client, &owner, recipients, shares);
        client.execute_split(&caller, &id, &token, &5_000i128);

        let token_client = soroban_sdk::token::Client::new(&env, &token);
        assert_eq!(token_client.balance(&r1), 5000);
    }

    #[test]
    fn test_update_split() {
        let env = Env::default();
        let (_, client) = setup(&env);
        let owner = Address::generate(&env);
        let r1 = Address::generate(&env);
        let r2 = Address::generate(&env);

        let mut recipients = Vec::new(&env);
        recipients.push_back(r1.clone());

        let mut shares = Vec::new(&env);
        shares.push_back(10000u32);

        let id = make_split(&env, &client, &owner, recipients, shares);

        let mut new_recipients = Vec::new(&env);
        new_recipients.push_back(r1.clone());
        new_recipients.push_back(r2.clone());

        let mut new_shares = Vec::new(&env);
        new_shares.push_back(5000u32);
        new_shares.push_back(5000u32);

        client.update_split(&owner, &id, &new_recipients, &new_shares);

        let split = client.get_split(&id);
        assert_eq!(split.recipients.len(), 2);
        assert_eq!(split.shares.get(0).unwrap(), 5000u32);
        assert_eq!(split.shares.get(1).unwrap(), 5000u32);
    }

    #[test]
    fn test_deactivate_split() {
        let env = Env::default();
        let (_, client) = setup(&env);
        let owner = Address::generate(&env);
        let r1 = Address::generate(&env);

        let mut recipients = Vec::new(&env);
        recipients.push_back(r1);

        let mut shares = Vec::new(&env);
        shares.push_back(10000u32);

        let id = make_split(&env, &client, &owner, recipients, shares);
        assert!(client.get_split(&id).active);

        client.deactivate_split(&owner, &id);
        assert!(!client.get_split(&id).active);
    }

    #[test]
    fn test_reactivate_split() {
        let env = Env::default();
        let (_, client) = setup(&env);
        let owner = Address::generate(&env);
        let r1 = Address::generate(&env);

        let mut recipients = Vec::new(&env);
        recipients.push_back(r1);

        let mut shares = Vec::new(&env);
        shares.push_back(10000u32);

        let id = make_split(&env, &client, &owner, recipients, shares);
        client.deactivate_split(&owner, &id);
        assert!(!client.get_split(&id).active);

        client.reactivate_split(&owner, &id);
        assert!(client.get_split(&id).active);
    }

    #[test]
    fn test_multiple_splits_increment_counter() {
        let env = Env::default();
        let (_, client) = setup(&env);
        let owner = Address::generate(&env);
        let r1 = Address::generate(&env);

        let mut recipients = Vec::new(&env);
        recipients.push_back(r1);

        let mut shares = Vec::new(&env);
        shares.push_back(10000u32);

        let id1 = make_split(&env, &client, &owner, recipients.clone(), shares.clone());
        let id2 = make_split(&env, &client, &owner, recipients.clone(), shares.clone());
        let id3 = make_split(&env, &client, &owner, recipients, shares);

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
        assert_eq!(client.get_count(), 3);
    }

    #[test]
    fn test_get_recipients_and_shares() {
        let env = Env::default();
        let (_, client) = setup(&env);
        let owner = Address::generate(&env);
        let r1 = Address::generate(&env);
        let r2 = Address::generate(&env);

        let mut recipients = Vec::new(&env);
        recipients.push_back(r1.clone());
        recipients.push_back(r2.clone());

        let mut shares = Vec::new(&env);
        shares.push_back(6000u32);
        shares.push_back(4000u32);

        let id = make_split(&env, &client, &owner, recipients, shares);

        let r = client.get_recipients(&id);
        assert_eq!(r.len(), 2);

        let s = client.get_shares(&id);
        assert_eq!(s.get(0).unwrap(), 6000u32);
        assert_eq!(s.get(1).unwrap(), 4000u32);
    }

    #[test]
    #[should_panic(expected = "Shares must sum to 10000 basis points")]
    fn test_shares_must_sum_to_10000() {
        let env = Env::default();
        let (_, client) = setup(&env);
        let owner = Address::generate(&env);
        let r1 = Address::generate(&env);

        let mut recipients = Vec::new(&env);
        recipients.push_back(r1);

        let mut shares = Vec::new(&env);
        shares.push_back(5000u32);

        client.create_split(
            &owner,
            &String::from_str(&env, "Bad Split"),
            &recipients,
            &shares,
        );
    }

    #[test]
    #[should_panic(expected = "Recipients and shares length mismatch")]
    fn test_length_mismatch_panics() {
        let env = Env::default();
        let (_, client) = setup(&env);
        let owner = Address::generate(&env);
        let r1 = Address::generate(&env);
        let r2 = Address::generate(&env);

        let mut recipients = Vec::new(&env);
        recipients.push_back(r1);
        recipients.push_back(r2);

        let mut shares = Vec::new(&env);
        shares.push_back(10000u32);

        client.create_split(
            &owner,
            &String::from_str(&env, "Bad Split"),
            &recipients,
            &shares,
        );
    }

    #[test]
    #[should_panic(expected = "Split is not active")]
    fn test_execute_inactive_split_panics() {
        let env = Env::default();
        let (_, client) = setup(&env);
        let token = make_token(&env);
        let owner = Address::generate(&env);
        let caller = Address::generate(&env);
        let r1 = Address::generate(&env);

        mint(&env, &token, &caller, 1000);

        let mut recipients = Vec::new(&env);
        recipients.push_back(r1);

        let mut shares = Vec::new(&env);
        shares.push_back(10000u32);

        let id = make_split(&env, &client, &owner, recipients, shares);
        client.deactivate_split(&owner, &id);
        client.execute_split(&caller, &id, &token, &1000i128);
    }

    #[test]
    #[should_panic(expected = "Amount must be greater than zero")]
    fn test_zero_amount_panics() {
        let env = Env::default();
        let (_, client) = setup(&env);
        let token = make_token(&env);
        let owner = Address::generate(&env);
        let caller = Address::generate(&env);
        let r1 = Address::generate(&env);

        let mut recipients = Vec::new(&env);
        recipients.push_back(r1);

        let mut shares = Vec::new(&env);
        shares.push_back(10000u32);

        let id = make_split(&env, &client, &owner, recipients, shares);
        client.execute_split(&caller, &id, &token, &0i128);
    }

    #[test]
    #[should_panic(expected = "Not the split owner")]
    fn test_non_owner_cannot_deactivate() {
        let env = Env::default();
        let (_, client) = setup(&env);
        let owner = Address::generate(&env);
        let attacker = Address::generate(&env);
        let r1 = Address::generate(&env);

        let mut recipients = Vec::new(&env);
        recipients.push_back(r1);

        let mut shares = Vec::new(&env);
        shares.push_back(10000u32);

        let id = make_split(&env, &client, &owner, recipients, shares);
        client.deactivate_split(&attacker, &id);
    }
}
