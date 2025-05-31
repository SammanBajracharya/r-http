use std::io::{self, BufRead, Write};
use std::net::UdpSocket;

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut line = String::new();

    let socket = UdpSocket::bind("127.0.0.1:0")?;
    socket.connect("127.0.0.1:42069")?;

    loop {
        print!("> ");
        io::stdout().flush()?;
        reader.read_line(&mut line)?;

        if let Err(e) = socket.send(line.as_bytes()) {
            eprintln!("error sending UDP message: {}", e);
        }
        line.clear();
    }
}
