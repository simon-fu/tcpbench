use std::{sync::Arc, time::{Duration, Instant}};

use anyhow::{Result, Context};
use bytes::BufMut;
use rust_tcp::{now_millis, Pacer};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}};

use crate::args::ServerArgs;


pub async fn run(args: ServerArgs) -> Result<()> {
    
    let args = Arc::new(args);

    let listener = TcpListener::bind(&args.addr).await
    .with_context(||format!("fail to listen at [{}]", args.addr))?;

    println!("listening at [{}]", args.addr);
    println!("press Enter to send packets...");

    let mut sockets = accept_sockets(listener).await?;

    let connections = sockets.len();
    println!("total connections [{}]", connections);
    println!("sending packets [{}] at rate [{}] in seconds [{:?}]...", args.pps * args.secs, args.pps, args.secs);

    let start_time = Instant::now();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel(); 
    while let Some(mut socket) = sockets.pop() {
        let args = args.clone();
        let tx = tx.clone();
        tokio::spawn(async move { 
            let r = send_packets(&mut socket, args).await;
            let _r = tx.send(r);
            // tokio::time::sleep(Duration::MAX/4).await; // sleep forever in case of closing socket 
        });
    }

    for _ in 0..connections {
        let result  = rx.recv().await.with_context(||"fail to waiting for done")?;
        result?;
    }

    println!("elapsed [{:?}]", start_time.elapsed());
    println!("done.");

    Ok(())
}

async fn send_packets(socket: &mut TcpStream, args: Arc<ServerArgs>) -> Result<()> { 
    let packets = args.pps * args.secs;
    let mut buf = vec![0_u8; args.packet_len];

    let pacer = Pacer::new(args.pps);
    for n in 0..packets {
        if let Some(d) = pacer.get_sleep_duration(n) {
            tokio::time::sleep(d).await;
        }

        {
            let cursor = &mut &mut buf[..];
            let ts = now_millis();
            cursor.put_u32((args.packet_len - 4) as u32);
            cursor.put_i64(ts);
            cursor.put_u64(n);
            // println!("aaa: sent No.{} packets, ts {}", n, ts);
        }

        // socket.write_all(&buf[..]).await?;
        socket.write_all_buf(&mut &buf[..]).await?;
        
    }

    println!("aaa: sent {} packets", packets);

    // write last packet
    {
        let cursor = &mut &mut buf[..];
        cursor.put_u32(0);
    }

    // socket.write_all(&buf).await?;
    socket.write_all_buf(&mut &buf[..]).await?;

    socket.flush().await?;

    Ok(())
}

async fn accept_sockets(listener: TcpListener) -> Result<Vec<TcpStream>> { 
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);

    let task = tokio::spawn(async move {
        let mut sockets = Vec::new();
        let mut print_n = 0;
        let mut print_time = Instant::now() + Duration::from_millis(1000);
        loop {
            tokio::select! {
                r = listener.accept() => {
                    let (socket, _remote_addr) = r.with_context(||"accept socket fail")?;
                    sockets.push(socket);
                },

                _r = rx.recv() => {
                    break;
                }

                _r = tokio::time::sleep_until(print_time.into()) => {
                    print_time = Instant::now() + Duration::from_millis(1000);
                    if print_n < sockets.len() {
                        print_n = sockets.len();
                        println!("accept connections [{}]", print_n);
                    }
                }
            }
        }
        Result::<_>::Ok(sockets)
    });

    // let _ = std::io::Read::read(&mut std::io::stdin(), &mut [0u8]).unwrap();
    let mut stdin = tokio::io::stdin();
    stdin.read(&mut [0u8]).await?;
    tx.send(()).await?;

    let sockets = task.await??;
    Ok(sockets)
}

