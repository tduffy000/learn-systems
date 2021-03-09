use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

pub type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Copy)]
pub struct Account {
    balance: f32,
    password_hash: u64,
}

impl Account {
    pub fn new(pass: &str) -> Account {
        let mut hasher = DefaultHasher::new();
        pass.hash(&mut hasher);
        let hash = hasher.finish();
        Account {
            balance: 0.0,
            password_hash: hash,
        }
    }

    fn hash_password(self, pass: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        pass.hash(&mut hasher);
        hasher.finish()
    }

    pub fn is_correct_password(self, entry: &str) -> bool {
        let hash = self.hash_password(entry);
        hash == self.password_hash
    }

    pub fn get_balance(self) -> f32 {
        self.balance
    }

    pub fn increment_balance(mut self, amount: f32) -> Result<f32> {
        if (self.balance + amount) < 0.0 {
            return Err("Can't have a negative balance".to_string());
        }
        self.balance += amount;
        Ok(self.balance)
    }

    pub fn change_password(mut self, new: &str) -> Result<()> {
        let hash = self.hash_password(new);
        if hash != self.password_hash {
            Err("Already used".to_string())
        } else {
            self.password_hash = hash;
            Ok(())
        }
    }
}

pub struct AccountStore {
    accounts: HashMap<String, Account>,
}

// implement drop()
impl AccountStore {
    pub fn new() -> AccountStore {
        AccountStore {
            accounts: HashMap::new(),
        }
    }

    pub fn get_account(&self, user: String) -> Result<Account> {
        match self.accounts.get(&user) {
            Some(account) => Ok(*account),
            None => Err("No such account".to_string()),
        }
    }

    pub fn update_account(mut self, user: String, account: Account) -> Result<Account> {
        match self.accounts.insert(user, account) {
            Some(acct) => Ok(acct),
            None => Err("No such account".to_string()),
        }
    }

    pub fn add_account(mut self, user: String, account: Account) -> Result<Account> {
        match self.accounts.insert(user, account) {
            Some(_) => Err("Account exists".to_string()),
            None => Ok(account),
        }
    }
}
