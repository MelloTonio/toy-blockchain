extern crate serde;
extern crate serde_json;
extern crate sha2;
extern crate time;

use serde_derive::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt::Write;
use std::time::SystemTime;

// Used to serialize and deserialize json
// https://serde.rs/derive.html
#[derive(Debug, Clone, Serialize)]
struct Transaction {
    sender: String,
    receiver: String,
    amount: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Header {
    timestamp: std::time::SystemTime,
    nonce: u32,
    pre_hash: String,
    merkle_root: String,
    difficulty: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Block {
    header: Header,
    count: u32,
    transactions: Vec<Transaction>,
}

pub struct Chain {
    records: HashMap<String, f32>,
    chain: Vec<Block>,
    current_transaction: Vec<Transaction>,
    difficulty: u32,
    miner_address: String,
    reward: f32,
}

impl Chain {
    pub fn new(miner_address: String, difficulty: u32) -> Chain {
        let mut chain = Chain {
            records: HashMap::new(),
            chain: Vec::new(),
            current_transaction: Vec::new(),
            difficulty,
            miner_address,
            reward: 100.0,
        };

        chain.generate_new_block();
        chain
    }

    pub fn new_transaction(&mut self, sender: String, receiver: String, amount: f32) -> bool {
        if self.check_transfer_availability(&sender, &receiver, amount) != true {
            println!("Unable to complete the transaction");
            return false;
        }

        self.current_transaction.push(Transaction {
            sender,
            receiver,
            amount,
        });

        true
    }

    pub fn last_hash(&self) -> String {
        let block = match self.chain.last() {
            Some(block) => block, // If exists at least one (last) block, use it
            None => return String::from_utf8(vec![48; 64]).unwrap(), // else, we're dealing with the genesis block and we must create the first hash
        };

        Chain::hash(&block.header)
    }

    pub fn update_difficulty(&mut self, difficulty: u32) -> bool {
        self.difficulty = difficulty;
        true
    }

    pub fn update_reward(&mut self, reward: f32) -> bool {
        self.reward = reward;
        true
    }

    pub fn generate_new_block(&mut self) -> bool {
        let header = Header {
            timestamp: SystemTime::now(),
            nonce: 0,
            merkle_root: String::new(),
            pre_hash: self.last_hash(),
            difficulty: self.difficulty,
        };

        let transaction_reward = Transaction {
            sender: String::from("Root"),
            receiver: self.miner_address.clone(),
            amount: self.reward,
        };

        let mut block = Block {
            header,
            count: 0,
            transactions: vec![],
        };

        // Miner reward
        block.transactions.push(transaction_reward);
        // All Block transactions
        block.transactions.append(&mut self.current_transaction);
        block.count = block.transactions.len() as u32;
        block.header.merkle_root = Chain::get_merkle(block.transactions.clone());
        Chain::proof_of_work(&mut block.header);

        // Add mined coins to the receiver address
        let receiver = &self.miner_address;
        match self.records.get_mut(receiver) {
            Some(_val) => {
                *self.records.get_mut(receiver).unwrap() += self.reward;
                println!("Added {} coins to address {}", self.reward, receiver);
            }
            None => {
                self.records.insert(receiver.to_string(), self.reward);
                println!("Added {} coins to address {}", self.reward, receiver);
            }
        }

        println!("{:#?}", &block);
        self.chain.push(block);
        true
    }

    fn get_merkle(current_transactions: Vec<Transaction>) -> String {
        let mut merkle = Vec::new();

        for transaction in &current_transactions {
            let hash = Chain::hash(transaction);
            merkle.push(hash);
        }

        if merkle.len() % 2 == 1 {
            let last = merkle.last().cloned().unwrap();
            merkle.push(last);
        }

        while merkle.len() > 1 {
            // Get the next two (first) hashes
            let mut hash1 = merkle.remove(0);
            let mut hash2 = merkle.remove(0);

            // Creates a hash based on the two previous hashes
            hash1.push_str(&mut hash2);
            let mergedHash = Chain::hash(&hash1);

            // Put it back on the merkle_root vector
            merkle.push(mergedHash);
        }

        merkle.pop().unwrap()
    }

    pub fn proof_of_work(header: &mut Header) {
        loop {
            let hash = Chain::hash(header);
            println!("hash: {}", hash);
            let slice = &hash[..header.difficulty as usize];
            println!("slice: {}", slice);
            match slice.parse::<u32>() {
                Ok(val) => {
                    println!("val: {}", val);
                    if val != 0 {
                        header.nonce += 1;
                    } else {
                        println!("Block hash: {}", hash);
                        break;
                    }
                }
                Err(_) => {
                    header.nonce += 1;
                    continue;
                }
            };
        }
    }

    // Generic T here will be a type of serde.Serialize
    pub fn hash<T: serde::Serialize>(item: &T) -> String {
        let input = serde_json::to_string(&item).unwrap();
        let mut hasher = Sha256::default();

        hasher.update(input.as_bytes());
        let res = hasher.finalize();

        Chain::hex_to_string(&res[..])
    }

    pub fn hex_to_string(vec_res: &[u8]) -> String {
        let mut s = String::new();

        for b in vec_res {
            write!(&mut s, "{:x}", b).expect("unable to write")
        }

        s
    }

    // TODO: separate in two different functions (VALIDATE & TRANSFER)
    pub fn check_transfer_availability(
        &mut self,
        sender: &String,
        receiver: &String,
        amount: f32,
    ) -> bool {
        // Check if sender exists and has sufficient balance
        match self.records.get(sender) {
            Some(val) => {
                if val.clone() < amount {
                    println!("insufficient balance");
                    return false;
                }
            }
            None => println!("Sender not found!"),
        }

        // Remove the amount from sender current balance
        *self.records.get_mut(sender).unwrap() -= amount;

        // Add value in the receiver address
        match self.records.get_mut(receiver) {
            Some(_val) => {
                *self.records.get_mut(receiver).unwrap() += amount;
            }
            None => {
                self.records.insert(receiver.to_string(), amount);
            }
        }
        true
    }
}
