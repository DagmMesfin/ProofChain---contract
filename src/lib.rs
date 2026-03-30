#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Owner(String),
    Creator(String),
    LicenseType(String),
    Price(String),
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum TxType {
    Mint = 0,
    Exclusive = 1,
    NonExclusive = 2,
}

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    LicenseAlreadyExists = 1,
    LicenseNotFound = 2,
    NotCurrentOwner = 3,
}

#[contract]
pub struct ProofChainContract;

#[contractimpl]
impl ProofChainContract {
    pub fn mint(
        env: Env,
        license_id: String,
        creator: Address,
        license_type: u32,
        price: i128,
    ) -> Result<(), ContractError> {
        creator.require_auth();

        let owner_key = DataKey::Owner(license_id.clone());
        if env.storage().persistent().has(&owner_key) {
            return Err(ContractError::LicenseAlreadyExists);
        }

        env.storage().persistent().set(&owner_key, &creator);
        env.storage()
            .persistent()
            .set(&DataKey::Creator(license_id.clone()), &creator);
        env.storage()
            .persistent()
            .set(&DataKey::LicenseType(license_id.clone()), &license_type);
        env.storage()
            .persistent()
            .set(&DataKey::Price(license_id.clone()), &price);

        env.events().publish(
            (symbol_short!("mint"), license_id),
            (TxType::Mint as u32, creator, price),
        );

        Ok(())
    }

    pub fn transfer_exclusive(
        env: Env,
        license_id: String,
        seller: Address,
        buyer: Address,
        price: i128,
    ) -> Result<(), ContractError> {
        seller.require_auth();

        let owner_key = DataKey::Owner(license_id.clone());
        let current_owner = env
            .storage()
            .persistent()
            .get::<_, Address>(&owner_key)
            .ok_or(ContractError::LicenseNotFound)?;

        if current_owner != seller {
            return Err(ContractError::NotCurrentOwner);
        }

        env.storage().persistent().set(&owner_key, &buyer);
        env.storage()
            .persistent()
            .set(&DataKey::Price(license_id.clone()), &price);

        env.events().publish(
            (symbol_short!("sale"), license_id),
            (TxType::Exclusive as u32, seller, buyer, price),
        );

        Ok(())
    }

    pub fn record_nonexclusive(
        env: Env,
        license_id: String,
        seller: Address,
        buyer: Address,
        price: i128,
    ) -> Result<(), ContractError> {
        seller.require_auth();

        let owner_key = DataKey::Owner(license_id.clone());
        let current_owner = env
            .storage()
            .persistent()
            .get::<_, Address>(&owner_key)
            .ok_or(ContractError::LicenseNotFound)?;

        if current_owner != seller {
            return Err(ContractError::NotCurrentOwner);
        }

        env.events().publish(
            (symbol_short!("sale"), license_id),
            (TxType::NonExclusive as u32, seller, buyer, price),
        );

        Ok(())
    }

    pub fn owner_of(env: Env, license_id: String) -> Result<Address, ContractError> {
        env.storage()
            .persistent()
            .get::<_, Address>(&DataKey::Owner(license_id))
            .ok_or(ContractError::LicenseNotFound)
    }

    pub fn creator_of(env: Env, license_id: String) -> Result<Address, ContractError> {
        env.storage()
            .persistent()
            .get::<_, Address>(&DataKey::Creator(license_id))
            .ok_or(ContractError::LicenseNotFound)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address};

    fn lic(env: &Env) -> String {
        String::from_str(env, "license-001")
    }

    #[test]
    fn mint_sets_owner_and_creator() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, ProofChainContract);
        let client = ProofChainContractClient::new(&env, &contract_id);

        let creator = Address::generate(&env);
        client.mint(&lic(&env), &creator, &1u32, &1000i128);

        let owner = client.owner_of(&lic(&env));
        let creator_from_chain = client.creator_of(&lic(&env));

        assert_eq!(owner, creator);
        assert_eq!(creator_from_chain, creator);
    }

    #[test]
    fn exclusive_transfer_updates_owner() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, ProofChainContract);
        let client = ProofChainContractClient::new(&env, &contract_id);

        let seller = Address::generate(&env);
        let buyer = Address::generate(&env);

        client.mint(&lic(&env), &seller, &1u32, &1000i128);
        client.transfer_exclusive(&lic(&env), &seller, &buyer, &2500i128);

        let owner = client.owner_of(&lic(&env));
        assert_eq!(owner, buyer);
    }

    #[test]
    fn nonexclusive_does_not_change_owner() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, ProofChainContract);
        let client = ProofChainContractClient::new(&env, &contract_id);

        let seller = Address::generate(&env);
        let buyer = Address::generate(&env);

        client.mint(&lic(&env), &seller, &2u32, &500i128);
        client.record_nonexclusive(&lic(&env), &seller, &buyer, &100i128);

        let owner = client.owner_of(&lic(&env));
        assert_eq!(owner, seller);
    }
}
