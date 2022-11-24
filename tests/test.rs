use std::io::Write;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;

#[test]
fn test() {
    for _ in 0..1 {
        if let Ok(mut stream) = TcpStream::connect("127.0.0.1:6379") {
            println!("stream connected");

            /* stream
            .write_all(b"*2\r\n$4\r\necho\r\n$5\r\nworld\r\n")
            .expect("cannot send to server"); */
            /* stream
            .write_all(b"*1\r\n$4\r\nping\r\n")
            .expect("cannot send to server"); */
            /* stream
            .write_all(b"*3\r\n$3\r\nset\r\n$1\r\na\r\n$1\r\nb\r\n")
            .expect("cannot send to server"); */
            /* stream
            .write_all(b"*5\r\n$3\r\nset\r\n$1\r\na\r\n$1\r\nb\r\n$2\r\npx\r\n$5\r\n10000\r\n")
            .expect("cannot send to server"); */
            stream
                .write_all(b"*3\r\n$3\r\nget\r\n$1\r\na\r\n")
                .expect("cannot send to server");
            stream.flush().expect("cannot flush");

            handle_connection(&mut stream);
        } else {
            println!("could not connect");
        }
    }
}

fn handle_connection(mut stream: &TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let buffer: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    if let Some(value) = buffer.get(0) {
        println!("Read: {:#?}", buffer);
    }
}
