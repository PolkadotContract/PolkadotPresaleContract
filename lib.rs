#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod token_factory {
    use token_contract::TokenContractRef;
    use token_contract::PSP22Metadata;
    use ink::storage::Mapping as StorageHashMap;
    use ink::prelude::string::String;
    use ink::ToAccountId;
    use ink::env::hash::Blake2x256;



    #[ink(storage)]
    pub struct TokenFactory {
        tokens: StorageHashMap<AccountId, TokenContractRef>,
        owner: AccountId,
        fee: Balance,
        other_contract_code_hash: Hash
    }

    impl TokenFactory {
        #[ink(constructor)]
        pub fn new(other_contract_code_hash: Hash, fee: Balance) -> Self {
            let caller = Self::env().caller();

            Self {
                tokens: StorageHashMap::default(),
                owner: caller,
                fee,
                other_contract_code_hash
            }
        }

        #[ink(message)]
        pub fn get_fee(&self) -> Balance {
            self.fee
        }

        #[ink(message)]
        pub fn create_token(
            &mut self,
            initial_supply: Balance,
            name: String,
            symbol: String,
            decimals: u8,
            logo_uri: String, // New parameter for logo URI
        ) -> AccountId {
            let transferred_fee = self.env().transferred_value();
            if transferred_fee < self.fee {
                panic!("Insufficient fee, required: 10 Polkadot tokens");
            }

            let timestamp = Self::env().block_timestamp();
            let input = timestamp.to_le_bytes();
            

            let salt = Self::env().hash_encoded::<Blake2x256, _>(&input);

            let token_contract = TokenContractRef::new(initial_supply, Some(name), Some(symbol), decimals, Some(logo_uri), self.env().caller())
                .code_hash(self.other_contract_code_hash)
                .endowment(0)
                .salt_bytes(&salt.as_ref()[..4])
                .instantiate();


            let token_address = token_contract.to_account_id();

            self.tokens.insert(token_address, &token_contract);

            token_address
        }

        #[ink(message)]
        pub fn withdraw(&mut self, amount: u128) -> Result<(), String> {
            // Check if the caller is the owner (optional: remove or modify)
            let caller = self.env().caller();
            if caller != self.owner {
                return Err("Only the owner can withdraw".into());
            }

            // Ensure the contract has enough balance
            let contract_balance = self.env().balance();
            if contract_balance < amount {
                return Err("Insufficient balance".into());
            }

            // Perform the transfer of native tokens from the contract to the caller
            let destination = caller;
            let _ =  Self::env().transfer(destination, amount);

            Ok(())
        }

        /// Check contract balance
        #[ink(message)]
        pub fn contract_balance(&self) -> u128 {
            self.env().balance()
        }

        #[ink(message)]
        pub fn get_token_info(&self, token_address: AccountId) -> (String, String, u8, String) {
            let token = self.tokens.get(token_address).expect("Token not found");
            (token.token_name().expect("undefined"), token.token_symbol().expect("undefined"), token.token_decimals(), token.token_logo_uri().expect("undefined"))
        }
    }
}