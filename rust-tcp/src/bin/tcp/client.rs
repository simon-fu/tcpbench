use std::{sync::Arc, time::Instant};

use anyhow::{Result, Context};
use bytes::{BytesMut, Buf};
use rust_tcp::{now_millis, Pacer, normalize_addr};
use tokio::{io::AsyncReadExt, net::TcpStream};

use crate::args::ClientArgs;


pub async fn run(mut args: ClientArgs) -> Result<()> {
    
    normalize_addr(&mut args.target, "11111")?;
    let args = Arc::new(args);

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let pacer = Pacer::new(args.cps as u64);
    let start_time = Instant::now();
    for n in 0..args.conns {
        if let Some(d) = pacer.get_sleep_duration(n as u64) {
            tokio::time::sleep(d).await;
        }

        let mut socket = TcpStream::connect(&args.target).await
        .with_context(||format!("fail to connect to [{}]", args.target))?;

        let tx = tx.clone();
        tokio::spawn(async move {
            let r = recv_packest(&mut socket).await;
            let _r = tx.send((n, r));
        });
    }

    println!("make connections [{}] in [{:?}]", args.conns, start_time.elapsed());

    let mut all_latency = Vec::new();
    for _ in 0..args.conns {
        let (n, r)  = rx.recv().await.with_context(||"fail to waiting for done")?;
        let latency = r.with_context(||format!("No.{} connection error", n))?;
        all_latency.push(latency);
    }

    {
        let mut min = i64::MAX;
        let mut max = i64::MIN;
        let mut sum = 0;
        let mut num = 0;

        for list in &all_latency {
            for l in list {
                let latency = *l;
                sum += latency;
                if latency < min {
                    min = latency;
                }
                if latency > max {
                    max = latency;
                }

                num += 1;
            }
        }

        let average = if num > 0 {
            (sum + sum-1) / num
        } else {
            0
        };

        println!("latency: num/min/max/average {:?}", (num, min, max, average))
    }

    println!("done.");

    Ok(())
}

async fn recv_packest(socket: &mut TcpStream) -> Result<Vec<i64>> {
    let mut latency = Vec::with_capacity(1_000_000);
    let mut buf = BytesMut::new(); 
    loop {
        while buf.len() < 4 {
            socket.read_buf(&mut buf).await?; 
        }

        let payload_len = buf.get_u32() as usize;
        
        if payload_len >= 8 { 
            while buf.len() < payload_len {
                socket.read_buf(&mut buf).await?;
            }

            let ts = buf.get_i64();
            let diff = now_millis() - ts;
            latency.push(diff);
            buf.advance((payload_len-8) as usize);
        } else {
            return Ok(latency)
        }
    }
}

