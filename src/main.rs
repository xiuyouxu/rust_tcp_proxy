use clap::{App, Arg};
use std::io;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::{Read,Write};

fn exchange(mut sock1: TcpStream, mut sock2: TcpStream) {
    let mut buf = [0u8; 1024];
    let mut bytes = sock1.read(&mut buf).unwrap();
    loop {
        sock2.write(&buf[..bytes]).unwrap();
        bytes = sock1.read(&mut buf).unwrap();
        if bytes <= 0 {
            break;
        }
    }
}

// 代理逻辑：为客户端连接创建一个对应的连接到目标地址，并开启两个线程，用于交换这两个连接的数据
fn proxy(stream: TcpStream, target_addr: String) -> io::Result<()> {
    let target_stream = TcpStream::connect(target_addr).unwrap();

    let src1 = stream.try_clone().unwrap();
    let src2 = stream.try_clone().unwrap();
    let dst1 = target_stream.try_clone().unwrap();
    let dst2 = target_stream.try_clone().unwrap();

    thread::spawn( || {
        exchange(src1, dst1);
    });
    thread::spawn( || {
        exchange(dst2, src2);
    });

    Ok(())
}

fn main() {
    let matches = App::new("proxy")
        .arg(Arg::with_name("proxy_address").help("Please specify proxy address"))
        .arg(Arg::with_name("target_address").help("Please specify target address"))
        // .arg(
        //     Arg::with_name("outfile")
        //         .short("o")
        //         .long("outfile")
        //         .takes_value(true)
        //         .help("Write output to a file instead of stdout"),
        // )
        .get_matches();

    let proxy_addr = matches
        .value_of("proxy_address")
        .unwrap_or_default()
        .to_string();
    let target_addr = matches
        .value_of("target_address")
        .unwrap_or_default()
        .to_string();

    // listen for proxy
    let listener = TcpListener::bind(proxy_addr).unwrap();

    // 为每个客户端连接开启一个处理线程
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let sc = stream.try_clone().unwrap();
        let ta = target_addr.clone();

        thread::spawn( || {
            proxy(sc, ta).unwrap_or_else(|error| eprintln!("{:?}", error))
        });
    }
}
