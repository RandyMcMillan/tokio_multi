use tokio_multi::*;

fn main() -> Result<(), Box<dyn Error>> {
    //for num in random_numbers() {
    //    println!("6:{:?}:{}", nanos().unwrap(), num);
    //    println!("7:{:?}:{}", millis().unwrap(), num);
    //}

    //println!("{}", CUSTOM_PORT);
    let cwd = env::current_dir().unwrap();
    let cwd_to_string_lossy: String = String::from(cwd.to_string_lossy());
    //println!("{}", cwd_to_string_lossy);
    let local_data_dir = data_local_dir();
    //println!("{}", local_data_dir.expect("REASON").display());
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
        //println!("echo_runtime.spawn: {:?}", nanos().unwrap());
        while let Some(mut sock) = rx.recv().await {
            //println!("35:{:?}\nrx.recv().await", nanos().unwrap());
            //prepended bytes are lost
            //103, 110, 111, 115, 116, 114
            let mut buf = prepend(vec![0u8; 512], &[b'g', b'n', b'o', b's', b't', b'r']);
            //println!("pre:buf.push:\n{:?}", &buf);
            //gnostr bytes
            //114, 116, 115, 111, 110, 103
            buf.push(b'r'); //last element 103
            buf.push(b't'); //last element 110
            buf.push(b's'); //last element 111
            buf.push(b'o'); //last element 115
            buf.push(b'n'); //last element 116
            buf.push(b'g'); //last element 114
                            //println!("post:buf.push:\n{:?}", &buf);
            tokio::spawn(async move {
                //println!("54:{:?}", nanos().unwrap());

                for num in random_numbers() {
                    //println!("57:nanos:{:?}:{}", nanos().unwrap(), num);
                    //println!("58:millis:{:?}:{}", millis().unwrap(), num);
                }

                //println!("pre:\n{:?}", &buf);
                loop {
                    for num in random_numbers() {
                        //println!("64:nanos:{:?}:{}", nanos().unwrap(), num);
                        //println!("65:millis:{:?}:{}", millis().unwrap(), num);
                    }

                    let bytes_read = sock.read(&mut buf).await.expect("failed to read request");

                    if bytes_read == 0 {
                        //println!("71:bytes_read = {}", bytes_read);
                        //println!("72:{:?}", nanos().unwrap());
                        return;
                    }
                    //println!("60:{:?}:{}", nanos().unwrap(), bytes_read);
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
                    //println!("{:?}:post:{:?}", nanos().unwrap(), new_buf);
                    let utf8_string = String::from_utf8(new_buf)
                        .map_err(|non_utf8| {
                            String::from_utf8_lossy(non_utf8.as_bytes()).into_owned()
                        })
                        .unwrap();

                    //println!("79:{:?}\n{}", nanos().unwrap(), utf8_string);
                    //buf.push(b'\n');
                }
            });
        }
    });

    // acceptor_runtime task is run in a blocking manner, so that our server
    // starts accepting new TCP connections. This task just accepts the
    // incoming TcpStreams and are sent to the sender half of the channel.
    acceptor_runtime.block_on(async move {
        println!("105:{:?}:acceptor_runtime is started", nanos().unwrap());
        let listener = match TcpListener::bind("127.0.0.1:8080").await {
            //8080
            Ok(l) => l,
            Err(e) => panic!("error binding TCP listener: {}", e),
        };

        loop {
            //println!(
            //    "101:{:?} acceptor_runtime: loop:listener:8080",
            //    nanos().unwrap()
            //);
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
    println!("116:{:?}:accept_conn", nanos().unwrap());
    match listener.accept().await {
        Ok((sock, _)) => Ok(sock),
        Err(e) => panic!("error accepting connection: {}", e),
    }
}
