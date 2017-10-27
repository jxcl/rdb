use std::string::String;
use std::io::{self, Write};

mod connection;

fn prompt() {
    print!("> ");
    io::stdout().flush().unwrap();
}

fn main() {
    let mut connection = connection::DbConnection::new();

    loop {
        let mut input = String::new();
        prompt();

        io::stdin().read_line(&mut input)
            .expect("Failed to read line");

        match connection.execute(&input) {
            Ok(connection::ConnectionResult::Output(output)) => println!("{}", output),
            Ok(connection::ConnectionResult::Empty) => (),
            Err(error) => println!("Error: {}", error),
        }
    }
}
