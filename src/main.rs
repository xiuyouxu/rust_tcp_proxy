use clap::{App, Arg};
use std::io;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;

// get socket connection info
fn stream_info(socket: &TcpStream) -> String {
    let peer_addr = socket.peer_addr().unwrap().to_string();
    let peer_addr = &peer_addr[..];
    let local_addr = socket.local_addr().unwrap().to_string();
    let local_addr = &local_addr[..];

    format!("{} <=> {}", local_addr, peer_addr)
}

// exchange data from sock1 to sock2, i.e. read data from sock1 and write it to sock2
fn exchange(mut sock1: TcpStream, mut sock2: TcpStream) {
    let mut buf = [0u8; 1024];
    loop {
        let bytes = sock1.read(&mut buf);
        match bytes {
            Ok(read_bytes) => {
                if read_bytes <= 0 {
                    break;
                }
                let v = sock2.write(&buf[..read_bytes]);
                match v {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Can not write data to {}: {}", stream_info(&sock2), e);
                        sock1
                            .shutdown(Shutdown::Both)
                            .expect(&format!("Failed to shutdown {}", stream_info(&sock1))[..]);
                        sock2
                            .shutdown(Shutdown::Both)
                            .expect(&format!("Failed to shutdown {}", stream_info(&sock2))[..]);
                    }
                }
            }
            Err(e) => {
                eprintln!("Can not read data from {}: {}", stream_info(&sock1), e);
                sock1
                    .shutdown(Shutdown::Both)
                    .expect(&format!("Failed to shutdown {}", stream_info(&sock1))[..]);
                sock2
                    .shutdown(Shutdown::Both)
                    .expect(&format!("Failed to shutdown {}", stream_info(&sock2))[..]);
            }
        }
    }
}

// connect target address for a client `stream` to get a `target_stream`,
// and spawn 2 threads for a client `stream`,
// one to read from `stream` and write to `target_stream`,
// the other to read from `target_stream` and write to `stream`
fn proxy(stream: TcpStream, target_addr: String) -> io::Result<()> {
    let target_stream = TcpStream::connect(target_addr);
    match target_stream {
        Ok(ts) => {
            let src1 = stream.try_clone().unwrap();
            let src2 = stream.try_clone().unwrap();
            let dst1 = ts.try_clone().unwrap();
            let dst2 = ts.try_clone().unwrap();

            thread::spawn(|| {
                exchange(src1, dst1);
            });
            thread::spawn(|| {
                exchange(dst2, src2);
            });
        }
        Err(e) => {
            eprintln!("Can not connect to target address: {}", e);
            stream
                .shutdown(Shutdown::Both)
                .expect("Failed to shutdown client connection to proxy");
            return Err(e);
        }
    }

    Ok(())
}

fn main() {
    // parse the arguments
    let matches = App::new("proxy")
        .arg(Arg::with_name("proxy_addr").help("Please set proxy address"))
        .arg(Arg::with_name("target_addr").help("Please set target address"))
        .get_matches();

    let proxy_addr = matches
        .value_of("proxy_addr")
        .unwrap_or_default()
        .to_string();
    let target_addr = matches
        .value_of("target_addr")
        .unwrap_or_default()
        .to_string();
    if proxy_addr.is_empty() || target_addr.is_empty() {
        eprintln!("Please set proxy address and target address:\n\tUsage: proxy <proxy address> <target address>\n\tExample: proxy 127.0.0.1:8080 127.0.0.1:80");
        std::process::exit(1);
    }

    // listen for proxy
    let listener = TcpListener::bind(proxy_addr).unwrap();

    // handle each client in a new thread in case of blocking the listener
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let sc = stream.try_clone().unwrap();
        let ta = target_addr.clone();

        thread::spawn(|| proxy(sc, ta).unwrap_or_else(|error| eprintln!("{:?}", error)));
    }
}
