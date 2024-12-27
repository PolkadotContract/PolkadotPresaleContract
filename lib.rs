#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod polkadot_presale_contract {
    use token_factory::TokenFactoryRef;
    use token_lock::TokenLockRef;
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
        start_time: Timestamp,
        end_time: Timestamp,
        creator: AccountId,
        contributors: Vec<AccountId>,
        is_finished: bool,
        is_successful: bool,
    }
    #[ink(storage)]
    pub struct PolkadotPresaleContract {
        projects: StorageHashMap<u32, Project>,
        last_project_id: u32,
        token_factory: TokenFactoryRef,
        token_lock: TokenLockRef,
    }

    impl PolkadotPresaleContract {
        #[ink(constructor)]
        pub fn new(token_factory_address: AccountId, token_lock_address: AccountId) -> Self {
            let token_factory: TokenFactoryRef = ink::env::call::FromAccountId::from_account_id(token_factory_address);
            let token_lock: TokenLockRef = ink::env::call::FromAccountId::from_account_id(token_lock_address);
            
            Self { token_factory, token_lock, last_project_id: 0, projects: StorageHashMap::new()}
        }

        #[ink(message)]
        #[allow(clippy::too_many_arguments)]
        pub fn create_presale(
            &mut self,
            max_supply: Balance,
            name: String,
            symbol: String,
            decimals: u8,
            logo_uri: String,
            lock_amount: Balance,
            lock_duartion: Timestamp,
            intended_raise_amount: Balance,
            start_time: Timestamp,
            end_time: Timestamp,
        ) {
            let project_id = self.last_project_id.checked_add(1).expect("Overflow detected in project_id calculation");
            self.last_project_id = project_id;

            let token_address = self.token_factory.create_token(max_supply, name, symbol, decimals, logo_uri);
            let _ = self.token_lock.create_lock(token_address, self.env().caller(), lock_amount, lock_duartion);

            let project = Project {
                token: token_address,
                total_presale_token_amount: max_supply.checked_sub(lock_amount).expect("error"),
                presaled_amount: 0,
                intended_raise_amount,
                raised_amount: 0,
                start_time,
                end_time,
                creator: self.env().caller(),
                contributors: Vec::new(),
                is_finished: false,
                is_successful: false,
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

            assert!(project.start_time <= self.time_now(), "Presale not started");
            assert!(project.end_time > self.time_now(), "Presale ended");

            let cost = self.calculate_price(project.presaled_amount, buy_token_amount);
            assert!(cost < self.env().transferred_value(), "Insufficient payment");
            assert!(project.presaled_amount.checked_add(buy_token_amount) < Some(project.total_presale_token_amount), "Insufficient amount");

            project.presaled_amount = project.presaled_amount.checked_add(buy_token_amount).expect("Invalid Operation");
            project.raised_amount = project.raised_amount.checked_add(cost).expect("Invalid Operation");
            project.contributors.push(caller);
        }

        #[ink(message)]
        pub fn finish_presale(
            &mut self,
            project_id: u32,
        ) {
            let mut project = self.projects.get(project_id).expect("Project not found");

            assert!(project.end_time > self.time_now(), "Presale not finished");
            project.is_finished = true;
            project.is_successful = project.intended_raise_amount / 3 <= project.raised_amount;

            // Add more logic here
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

        pub fn time_now(&self) -> Timestamp {
            self.env().block_timestamp()
        }
    }
}
