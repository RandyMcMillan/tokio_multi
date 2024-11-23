// rustc version 1.77.2
// tokio version 1.38.0, features: rt-multi-thread, net, io-util

use dirs::*;
use std::env;
use std::error::Error;

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::mpsc,
};

use std::time::{SystemTime, UNIX_EPOCH};

const CUSTOM_PORT: usize = 8000;

fn prepend<T>(v: Vec<T>, s: &[T]) -> Vec<T>
where
    T: Clone,
{
    let mut tmp: Vec<_> = s.to_owned();
    tmp.extend(v);
    tmp
}

// Pseudorandom number generator from the "Xorshift RNGs" paper by George Marsaglia.
//
// https://github.com/rust-lang/rust/blob/1.55.0/library/core/src/slice/sort.rs#L559-L573
pub fn random_numbers() -> impl Iterator<Item = u32> {
    let mut random = 92u32;
    std::iter::repeat_with(move || {
        random ^= random << 13;
        random ^= random >> 17;
        random ^= random << 5;
        random
    })
}

pub fn random_seed() -> u64 {
    RandomState::new().build_hasher().finish()
}

fn nanos() -> Result<(), Box<dyn Error>> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.subsec_nanos();

    // Prints 864479511, 455850730, etc.
    println!("Random number: {nanos}");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    //let _ = nanos();
    println!("{}", CUSTOM_PORT);
    let cwd = env::current_dir().unwrap();
    let cwd_to_string_lossy: String = String::from(cwd.to_string_lossy());
    println!("{}", cwd_to_string_lossy);
    let local_data_dir = data_local_dir();
    println!("{}", local_data_dir.expect("REASON").display());
    // Create a tokio runtime whose job is to simply accept new incoming TCP connections.
    let acceptor_runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_name("acceptor-pool")
        .enable_all()
        .build()?;

    // Create another tokio runtime whose job is only to write the response bytes to the outgoing TCP message.
    let echo_runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(8)
        .thread_name("echo-handler-pool")
        .enable_all()
        .build()?;

    // this channel is used to pass the TcpStream from acceptor_runtime task to
    // to echo_runtime task where the request handling is done.
    let (tx, mut rx) = mpsc::channel::<TcpStream>(CUSTOM_PORT.into());

    // The receiver part of the channel is moved inside a echo_runtime task.
    // This task simply writes the echo response to the TcpStreams coming through the
    // channel receiver.
    echo_runtime.spawn(async move {
        println!("echo_runtime.spawn");
        while let Some(mut sock) = rx.recv().await {
            println!("rx.recv().await");
            //prepended bytes are lost
            //103, 110, 111, 115, 116, 114
            let mut buf = prepend(vec![0u8; 512], &[b'g', b'n', b'o', b's', b't', b'r']);
            println!("pre:buf.push:\n{:?}", &buf);
            //gnostr bytes
            //114, 116, 115, 111, 110, 103
            buf.push(b'r'); //last element 103
            buf.push(b't'); //last element 110
            buf.push(b's'); //last element 111
            buf.push(b'o'); //last element 115
            buf.push(b'n'); //last element 116
            buf.push(b'g'); //last element 114
            println!("post:buf.push:\n{:?}", &buf);

            tokio::spawn(async move {
                /*loop {*/
                println!("pre:\n{:?}", &buf);
                loop {
                    let bytes_read = sock.read(&mut buf).await.expect("failed to read request");

                    if bytes_read == 0 {
                        println!("bytes_read = {}", bytes_read);
                        return;
                    }
                    println!("bytes_read = {}", bytes_read);
                    let mut new_buf = prepend(vec![0u8; 512], &buf);

                    new_buf.push(b'g'); //last element 32
                    new_buf.push(b'n'); //last element 32
                    new_buf.push(b'o'); //last element 32
                    new_buf.push(b's'); //last element 32
                    new_buf.push(b't'); //last element 32
                    new_buf.push(b'r'); //last element 32
                    sock.write_all(&new_buf[0..bytes_read + 3])
                        .await
                        .expect("failed to write response");
                    println!("post:\n{:?}", new_buf);
                    let utf8_string = String::from_utf8(new_buf)
                        .map_err(|non_utf8| String::from_utf8_lossy(non_utf8.as_bytes())
                        .into_owned())
                        .unwrap();
                    println!("{}", utf8_string);
                    //buf.push(b'\n');
                }
                /*}*/
            });
        }
    });

    // acceptor_runtime task is run in a blocking manner, so that our server
    // starts accepting new TCP connections. This task just accepts the
    // incoming TcpStreams and are sent to the sender half of the channel.
    acceptor_runtime.block_on(async move {
        println!("acceptor_runtime is started");
        let listener = match TcpListener::bind("127.0.0.1:8080").await {
            //8080
            Ok(l) => l,
            Err(e) => panic!("error binding TCP listener: {}", e),
        };

        loop {
            println!("acceptor_runtime loop:listener:8080");
            let sock = match accept_conn(&listener).await {
                Ok(stream) => stream,
                Err(e) => panic!("error reading TCP stream: {}", e),
            };
            let _ = tx.send(sock).await;
        }
    });

    Ok(())
}

async fn accept_conn(listener: &TcpListener) -> Result<TcpStream, Box<dyn Error>> {
    //loop {
    /*return*/
    println!("accept_conn");
    match listener.accept().await {
        Ok((sock, _)) => Ok(sock),
        Err(e) => panic!("error accepting connection: {}", e),
    } /*;*/
    //}
}
