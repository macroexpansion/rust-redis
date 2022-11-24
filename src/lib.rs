pub mod resp_parser;

use std::time::{Duration, Instant};

pub struct ValueWithExpiry {
    pub value: String,
    expires_at: Option<Instant>,
}

impl ValueWithExpiry {
    pub fn new(value: String, duration: Option<String>) -> Self {
        let expires_at: Option<Instant> = match duration {
            Some(value) => {
                let expires_at = Instant::now()
                    + Duration::from_millis(
                        value.parse::<u64>().expect("Unable to decipher duration"),
                    );
                Some(expires_at)
            }
            None => None,
        };

        println!("expires_at {expires_at:?}");

        Self { value, expires_at }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expiry_date) = self.expires_at {
            if expiry_date < Instant::now() {
                return true;
            }
            return false;
        }
        false
    }
}
