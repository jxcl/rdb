use std::collections::HashMap;
use std::string::String;

pub enum ConnectionResult {
    Output(String),
    Empty
}

pub struct DbConnection {
    connection: HashMap<String, String>,
}

impl DbConnection {
    pub fn new() -> DbConnection {
        DbConnection {
            connection: HashMap::new()
        }
    }

    pub fn execute(&mut self, input: &str) -> Result<ConnectionResult, &'static str> {
        let mut tokens = input.split_whitespace();

        let command = match tokens.next() {
            Some(comm) => comm,
            None => return Err("No command specified")
        };

        if command == "set" {
            let key = match tokens.next() {
                Some(k) => k,
                None => return Err("")
            };

            let value = match tokens.next() {
                Some(v) => v,
                None => return Err(""),
            };

            self.set(key, value);

            return Ok(ConnectionResult::Empty);

        } else if command == "get" {
            let key = match tokens.next() {
                Some(k) => k,
                None => return Err(""),
            };

            match self.get(key) {
                Some(output) => return Ok(ConnectionResult::Output(output.clone())),
                None => return Err("Key not found"),
            }
        } else {
            return Err("Unrecognized command");
        }
    }

    fn set(&mut self, key: &str, value: &str) {
        self.connection.insert(key.to_string(), value.to_string());
    }

    fn get(&self, key: &str) -> Option<&String> {
        self.connection.get(&key.to_string())
    }
}
