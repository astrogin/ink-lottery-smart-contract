#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod lottery {
    use ink::prelude::vec::Vec;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Lottery {
        // we will use it to pay some rewards for author of the lottery
        author: AccountId,
        // lottery round will be completed when date will be reached
        end_date: u64,
        // amount to by one ticket
        ticket_cost: u128,
        // participant wallets
        participants: Vec<AccountId>,
        // lottery balance
        prize_pool: Balance,
        // salt for random number generator
        salt: u64,
    }

    impl Lottery {
        #[ink(constructor)]
        pub fn new(ticket_cost: Option<u128>, end_date: Option<u64>) -> Self {
            // person who init the contract
            let author = Self::env().caller();
            // if amount not defined then keep it 100
            let ticket_cost = ticket_cost.unwrap_or(100);
            // if time not defined then keep it 7 days
            //get current block time
            let current_block_time: u64 = Self::env().block_timestamp();
            // count seconds in 7 days
            let seven_days_in_second: u64 = 7 * 24 * 60 * 60;
            let end_date = end_date.unwrap_or(current_block_time + seven_days_in_second);
            // keep participants empty when init
            let participants = Vec::default();
            Self { author, ticket_cost, end_date, participants, prize_pool: 0, salt: 1 }
        }

        #[ink(message, payable)]
        pub fn buy_ticket(&mut self) {
            // person who call the contract
            let caller = self.env().caller();
            // transferred amount from user
            let transferred_balance = self.env().transferred_value();
            //check if transferred amount equal ticket cost
            assert!(transferred_balance != self.ticket_cost, "Insufficient balance");
            // add amount to prize pool
            self.prize_pool = self.prize_pool + transferred_balance;
            // increase salt for random function
            self.salt += 1;
            // add caller to participants
            self.participants.push(caller);
        }

        #[ink(message)]
        pub fn get_price_pool(&mut self) -> Balance {
            self.prize_pool
        }

        #[ink(message)]
        pub fn get_time_left(&self) -> u64 {
            let current_time = self.env().block_timestamp();
            if current_time > self.end_date {
                return 0;
            }
            self.end_date - current_time
        }

        #[ink(message)]
        pub fn winner(&mut self) {
            let time_left = self.get_time_left();
            ink::env::debug_println!("time left {:?}.", time_left);
            //add salt for random function
            self.salt += 1;
            assert!(time_left > 0, "Time not exceeded");
            //generate random number
            let random_number = self.get_pseudo_random(self.participants.len() as u64) as usize;

            ink::env::debug_println!("random code {:?}.", random_number);

            // pick the winner
            let winner = self.participants.get(random_number).unwrap_or(&self.author);

            //transfer money to winner
            self.env().transfer(*winner, self.prize_pool).unwrap();

            //restart contract
            //clean up section
            self.prize_pool = 0;
            self.participants = Vec::new();
            let current_block_time: u64 = Self::env().block_timestamp();
            let seven_days_in_second: u64 = 7 * 24 * 60 * 60;
            self.end_date = current_block_time + seven_days_in_second;
            self.salt = 1;
        }

        #[ink(message)]
        pub fn set_code(&mut self, code_hash: [u8; 32]) {
            ink::env::set_code_hash(&code_hash).unwrap_or_else(|err| {
                panic!("Failed to `set_code_hash` to {code_hash:?} due to {err:?}")
            });
            ink::env::debug_println!("Switched code hash to {:?}.", code_hash);
        }

        pub fn get_pseudo_random(&mut self, max_value: u64) -> u64 {
            //get current block timestamp
            let seed = self.env().block_timestamp();
            // get random number in specific range
            let number = (seed + self.salt) % (max_value + 1);
            number
        }
    }
}
