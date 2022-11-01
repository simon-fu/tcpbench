use std::{sync::Arc, time::Instant, collections::VecDeque};

use anyhow::{Result, Context};
use bytes::{BytesMut, Buf};
use rust_tcp::{now_millis, Pacer, normalize_addr, Latency};
use tokio::{io::AsyncReadExt, net::TcpStream};

use crate::args::ClientArgs;


pub async fn run(mut args: ClientArgs) -> Result<()> {
    
    normalize_addr(&mut args.target, "11111")?;
    let args = Arc::new(args);

    let start_time = Instant::now();
    let sockets = make_connections(args.clone()).await?; 
    println!("make connections [{}] in [{:?}]", args.conns, start_time.elapsed());

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    for (n, mut socket) in sockets.into_iter().enumerate() {
        let tx = tx.clone();
        let args = args.clone();
        tokio::spawn(async move {
            let r = recv_packest(&mut socket, n, args.print_latency).await;
            let _r = tx.send((n, r));
            Result::<_>::Ok(())
        });
    }

    let mut all_latency = Vec::with_capacity(args.conns as usize);
    for _ in 0..args.conns {
        let (n, r)  = rx.recv().await.with_context(||"fail to waiting for done")?;
        let latency = r.with_context(||format!("No.{} connection error", n))?;
        all_latency.push(latency);
    }

    {
        let latency = Latency::from_iter(all_latency.iter());
        println!("latency: num/min/max/average {:?}", (latency.num(), latency.min(), latency.max(), latency.average()))
    }

    println!("done.");

    Ok(())
}

async fn recv_packest(socket: &mut TcpStream, no: usize, print_latency: bool) -> Result<Latency> {
    let mut latency = Latency::new();
    let mut buf = BytesMut::new(); 
    loop {
        while buf.len() < 4 {
            socket.read_buf(&mut buf).await?; 
        }

        let payload_len = buf.get_u32() as usize;
        
        if payload_len >= (8+8) { 
            while buf.len() < payload_len {
                socket.read_buf(&mut buf).await?;
            }

            let ts = buf.get_i64();
            let _no = buf.get_u64();

            let diff = now_millis() - ts;
            latency.observe(diff);
            buf.advance((payload_len-(8+8)) as usize);

            if print_latency {
                println!("conn {}: {} ms", no, diff);
            }
        } else {
            return Ok(latency)
        }
    }
}

async fn make_connections(args: Arc<ClientArgs>) -> Result<Vec<TcpStream>> {
    let mut sockets = Vec::with_capacity(args.conns as usize);
    let mut tasks = VecDeque::with_capacity(args.conns as usize);

    let pacer = Pacer::new(args.cps as u64);

    let mut n = 0;
    while n < args.conns {
        if let Some(d) = pacer.get_sleep_duration(n as u64) {
            if let Some(h) = tasks.pop_front() {
                let socket = h.await??;
                sockets.push(socket);
                continue;
            } else {
                tokio::time::sleep(d).await;
            }
        }

        let args = args.clone();

        let h = tokio::spawn(async move {
            TcpStream::connect(&args.target).await
            .with_context(||format!("fail to connect to [{}]", args.target))
        });
        tasks.push_back(h);
        n += 1;
    }

    while let Some(h) = tasks.pop_front() {
        let socket = h.await??;
        sockets.push(socket);
    }

    Ok(sockets)
}

