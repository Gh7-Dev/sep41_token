#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype,
    token::Interface as TokenInterface,
    Address, Env, String, MuxedAddress
};

// ── Storage keys ──────────────────────────────────────────────────────────────
#[contracttype]
pub enum DataKey {
    Balance(Address),
    Allowance(Address, Address),
    Admin,
    TotalSupply,
    Decimals,
    Name,
    Symbol,
}

// ── Contract ──────────────────────────────────────────────────────────────────
#[contract]
pub struct Token;

// ── Helpers ───────────────────────────────────────────────────────────────────
fn get_balance(env: &Env, addr: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Balance(addr.clone()))
        .unwrap_or(0)
}

fn set_balance(env: &Env, addr: &Address, amount: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::Balance(addr.clone()), &amount);
}

fn get_allowance(env: &Env, owner: &Address, spender: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Allowance(owner.clone(), spender.clone()))
        .unwrap_or(0)
}

fn set_allowance(env: &Env, owner: &Address, spender: &Address, amount: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::Allowance(owner.clone(), spender.clone()), &amount);
}

fn get_total_supply(env: &Env) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::TotalSupply)
        .unwrap_or(0)
}

fn set_total_supply(env: &Env, amount: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::TotalSupply, &amount);
}

// ── initialize and mint only ──────────────────────────────────────────────────
#[contractimpl]
impl Token {
    pub fn initialize(
        env: Env,
        admin: Address,
        decimals: u32,
        name: String,
        symbol: String,
        initial_supply: i128,
    ) {
        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::Decimals, &decimals);
        env.storage().persistent().set(&DataKey::Name, &name);
        env.storage().persistent().set(&DataKey::Symbol, &symbol);
        set_balance(&env, &admin, initial_supply);
        set_total_supply(&env, initial_supply);
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        let admin: Address = env.storage()
            .persistent()
            .get(&DataKey::Admin)
            .unwrap();
        admin.require_auth();
        let balance = get_balance(&env, &to);
        set_balance(&env, &to, balance + amount);
        let supply = get_total_supply(&env);
        set_total_supply(&env, supply + amount);
    }
}

// ── SEP-41 interface ──────────────────────────────────────────────────────────
#[contractimpl]
impl TokenInterface for Token {

    fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        get_allowance(&env, &from, &spender)
    }

    fn approve(
        env: Env,
        from: Address,
        spender: Address,
        amount: i128,
        expiration_ledger: u32,
    ) {
        from.require_auth();
        let _ = expiration_ledger;
        set_allowance(&env, &from, &spender, amount);
    }

    fn balance(env: Env, id: Address) -> i128 {
        get_balance(&env, &id)
    }

    fn transfer(env: Env, from: Address, to: MuxedAddress, amount: i128) {
        from.require_auth();
        let from_balance = get_balance(&env, &from);
        if from_balance < amount {
            panic!("insufficient balance");
        }
        set_balance(&env, &from, from_balance - amount);
        let to_addr: Address = to.address();
        let to_balance = get_balance(&env, &to_addr);
        set_balance(&env, &to_addr, to_balance + amount);
    }

    fn transfer_from(
        env: Env,
        spender: Address,
        from: Address,
        to: Address,
        amount: i128,
    ) {
        spender.require_auth();
        let allowance = get_allowance(&env, &from, &spender);
        if allowance < amount {
            panic!("insufficient allowance");
        }
        set_allowance(&env, &from, &spender, allowance - amount);
        let from_balance = get_balance(&env, &from);
        if from_balance < amount {
            panic!("insufficient balance");
        }
        set_balance(&env, &from, from_balance - amount);
        let to_balance = get_balance(&env, &to);
        set_balance(&env, &to, to_balance + amount);
    }

    fn burn(env: Env, from: Address, amount: i128) {
        from.require_auth();
        let balance = get_balance(&env, &from);
        if balance < amount {
            panic!("insufficient balance");
        }
        set_balance(&env, &from, balance - amount);
        let supply = get_total_supply(&env);
        set_total_supply(&env, supply - amount);
    }

    fn burn_from(env: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth();
        let allowance = get_allowance(&env, &from, &spender);
        if allowance < amount {
            panic!("insufficient allowance");
        }
        set_allowance(&env, &from, &spender, allowance - amount);
        let balance = get_balance(&env, &from);
        if balance < amount {
            panic!("insufficient balance");
        }
        set_balance(&env, &from, balance - amount);
        let supply = get_total_supply(&env);
        set_total_supply(&env, supply - amount);
    }

    fn decimals(env: Env) -> u32 {
        env.storage().persistent().get(&DataKey::Decimals).unwrap()
    }

    fn name(env: Env) -> String {
        env.storage().persistent().get(&DataKey::Name).unwrap()
    }

    fn symbol(env: Env) -> String {
        env.storage().persistent().get(&DataKey::Symbol).unwrap()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env, Address, String};

    fn setup() -> (Env, TokenClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(Token, ());
        let client = TokenClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(
            &admin,
            &7u32,
            &String::from_str(&env, "BlockTicket"),
            &String::from_str(&env, "BKT"),
            &1_000_000,
        );
        (env, client, admin)
    }

    #[test]
    fn test_mint() {
        let (env, client, _admin) = setup();
        let user = Address::generate(&env);
        client.mint(&user, &5000);
        assert_eq!(client.balance(&user), 5000);
    }

    #[test]
    fn test_burn() {
        let (env, client, _admin) = setup();
        let user = Address::generate(&env);
        client.mint(&user, &5000);
        client.burn(&user, &2000);
        assert_eq!(client.balance(&user), 3000);
    }

    #[test]
    fn test_burn_from() {
        let (env, client, _admin) = setup();
        let owner   = Address::generate(&env);
        let spender = Address::generate(&env);
        client.mint(&owner, &5000);
        client.approve(&owner, &spender, &3000, &100);
        client.burn_from(&spender, &owner, &2000);
        assert_eq!(client.balance(&owner), 3000);
        assert_eq!(client.allowance(&owner, &spender), 1000);
    }

    #[test]
    fn test_transfer_from() {
        let (env, client, _admin) = setup();
        let owner     = Address::generate(&env);
        let spender   = Address::generate(&env);
        let recipient = Address::generate(&env);
        client.mint(&owner, &5000);
        client.approve(&owner, &spender, &3000, &100);
        client.transfer_from(&spender, &owner, &recipient, &2000);
        assert_eq!(client.balance(&owner),     3000);
        assert_eq!(client.balance(&recipient), 2000);
        assert_eq!(client.allowance(&owner, &spender), 1000);
    }
}