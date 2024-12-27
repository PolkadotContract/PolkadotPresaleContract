#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod hydra_contracts {
    use token_factory::TokenFactoryRef;
    // use token_contract::TokenContractRef;
    use ink::ToAccountId;
    use ink::storage::{
        Mapping as StorageHashMap
    };
    use ink::prelude::string::String;
    use ink::prelude::{
        vec::Vec,
    };

    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(
        feature = "std",
        derive(ink::storage::traits::StorageLayout)
    )]
    pub struct Project {
        token: AccountId,
        total_presale_token_amount: Balance,
        presaled_amount: Balance,
        intended_raise_amount: Balance,
        raised_amount: Balance,
        start_time: u64,
        end_time: u64,
        creator: AccountId,
        contributors: Vec<AccountId>,
    }
    #[ink(storage)]
    pub struct HydraContracts {
        projects: StorageHashMap<u32, Project>,
        last_project_id: u32,
        token_factory: TokenFactoryRef,
    }

    impl HydraContracts {
        #[ink(constructor)]
        pub fn new(token_factory_address: AccountId) -> Self {
            let token_factory: TokenFactoryRef = ink::env::call::FromAccountId::from_account_id(token_factory_address);
            Self { token_factory, last_project_id: 0, projects: StorageHashMap::new()}
        }

        #[ink(message)]
        #[allow(clippy::too_many_arguments)]
        pub fn create_presale(
            &mut self,
            initial_supply: Balance,
            name: String,
            symbol: String,
            decimals: u8,
            logo_uri: String,
            total_presale_token_amount: Balance,
            intended_raise_amount: Balance,
            start_time: u64,
            end_time: u64,
        ) {
            let project_id = self.last_project_id.checked_add(1).expect("Overflow detected in project_id calculation");
            self.last_project_id = project_id;

            let token_address = self.token_factory.create_token(initial_supply, name, symbol, decimals, logo_uri);

            let project = Project {
                token: token_address,
                total_presale_token_amount,
                presaled_amount: 0,
                intended_raise_amount,
                raised_amount: 0,
                start_time,
                end_time,
                creator: self.env().caller(),
                contributors: Vec::new(),
            };

            self.projects.insert(project_id, &project);
        }


        #[ink(message)]
        pub fn join_project_presale(
            &mut self,
            project_id: u32,
            buy_token_amount: Balance,
        ) {
            let caller = self.env().caller();
            let mut project = self.projects.get(project_id).expect("Project not found");

            assert!(project.start_time <= self.env().block_timestamp(), "Presale not started");
            assert!(project.end_time > self.env().block_timestamp(), "Presale ended");

            let cost = self.calculate_price(project.presaled_amount, buy_token_amount);
            assert!(cost < self.env().transferred_value(), "Insufficient payment");
            assert!(project.presaled_amount.checked_add(buy_token_amount) < Some(project.total_presale_token_amount), "Insufficient amount");

            project.presaled_amount = project.presaled_amount.checked_add(buy_token_amount).expect("Invalid Operation");
            project.raised_amount = project.raised_amount.checked_add(cost).expect("Invalid Operation");
            project.contributors.push(caller);
        }

        #[ink(message)]
        pub fn calculate_price(
            &self,
            presaled_amount: Balance,
            buy_token_amount: Balance,
        ) -> Balance {
            let k: Balance = 1;
            let c: Balance = 0;

            let current_price = k.checked_mul(presaled_amount).unwrap_or(0).checked_add(c).unwrap_or(0);
            let next_price = k
                .checked_mul(presaled_amount.checked_add(buy_token_amount).unwrap_or(0))
                .unwrap_or(0)
                .checked_add(c)
                .unwrap_or(0);

            current_price
                .checked_add(next_price)
                .unwrap_or(0)
                .checked_mul(buy_token_amount)
                .unwrap_or(0)
                .checked_div(2)
                .unwrap_or(0)
        }

        #[ink(message)]
        pub fn get(&self) -> AccountId {
            self.token_factory.to_account_id()
        }
    }
}
