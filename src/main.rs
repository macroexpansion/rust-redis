use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::str;
use std::sync::{Arc, Mutex};

use redis_starter_rust::resp_parser::{DataType, RespParser};
use redis_starter_rust::ValueWithExpiry;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let storage = Arc::new(Mutex::new(HashMap::new()));

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    loop {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let storage_ref = Arc::clone(&storage);
                std::thread::spawn(move || loop {
                    let mut buffer = [0; 512];
                    let read = stream.read(&mut buffer).expect("cannot read from client");
                    println!("Read {read} byte(s)");
                    let resp_command: &str = str::from_utf8(&buffer)
                        .expect("invalid UTF-8")
                        .trim_end_matches(char::from(0));
                    println!("resp: {resp_command:?}");

                    let resp_response = parse_command(resp_command.as_bytes(), storage_ref.clone());

                    stream
                        .write_all(resp_response.as_bytes())
                        .expect("cannot send to client");
                    stream.flush().expect("cannot flush");
                });
            }
            Err(_err) => {
                println!("error");
                continue;
            }
        }
    }
}

fn parse_command(resp: &[u8], storage: Arc<Mutex<HashMap<String, ValueWithExpiry>>>) -> String {
    let mut parser = RespParser::new(resp);
    let parsed_commands = parser.parse();

    println!("{parsed_commands:?}");
    if parsed_commands.len() == 0 {
        return "+PONG\r\n".to_owned();
    }

    if let DataType::Array(_, commands) = parsed_commands.get(0).unwrap() {
        if let DataType::BulkString(_, value /* value has type &String*/) = &commands[0] {
            if *value == "PING".to_owned() || *value == "ping".to_owned() {
                return "+PONG\r\n".to_owned();
            }

            if *value == "ECHO".to_owned() || *value == "echo".to_owned() {
                if let DataType::BulkString(length, string) = &commands[1] {
                    return format!("${}\r\n{}\r\n", length, string);
                }
            }

            if *value == "SET".to_owned() || *value == "set".to_owned() {
                let key = if let DataType::BulkString(_, data) = &commands[1] {
                    data
                } else {
                    todo!()
                };
                let value = if let DataType::BulkString(_, data) = &commands[2] {
                    data
                } else {
                    todo!()
                };
                let duration = if let Some(DataType::BulkString(_, data)) = commands.get(4) {
                    Some(data.to_owned())
                } else {
                    None
                };
                let mut storage = storage.lock().expect("cannot acquire mutex");
                storage.insert(
                    key.to_owned(),
                    ValueWithExpiry::new(value.to_owned(), duration),
                );
                return "+OK\r\n".to_owned();
            }

            if *value == "GET".to_owned() || *value == "get".to_owned() {
                let key = if let DataType::BulkString(_, string) = &commands[1] {
                    string
                } else {
                    todo!()
                };
                let storage = storage.lock().expect("cannot acquire mutex");
                if let Some(value) = storage.get(key) {
                    if value.is_expired() {
                        return "$-1\r\n".to_owned();
                    }
                    return format!("${}\r\n{}\r\n", value.value.len(), value.value);
                }
                return "$-1\r\n".to_owned();
            }
        };
    };

    "-ERR no command found".to_owned()
}
