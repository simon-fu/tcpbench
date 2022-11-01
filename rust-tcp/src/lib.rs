
use std::time::{SystemTime, UNIX_EPOCH, Instant, Duration};
use anyhow::{Result, bail, anyhow};

pub fn now_millis() -> i64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let ms = since_the_epoch.as_secs() as i64 * 1000i64 + (since_the_epoch.subsec_nanos() as f64 / 1_000_000.0) as i64;
    ms
}

#[derive(Debug)]
pub struct Pacer {
    kick_time: Instant,
    rate: u64,
}

impl Pacer {
    pub fn new(rate: u64) -> Self {
        Pacer {
            kick_time: Instant::now(),
            rate,
        }
    }


    pub fn get_sleep_duration(&self, n: u64) -> Option<Duration> {
        if self.rate == 0 {
            return Some(Duration::from_millis(std::u64::MAX / 2));
        }

        let expect = 1000 * n / self.rate;
        let diff = expect as i64 - self.kick_time.elapsed().as_millis() as i64;
        if diff > 0 {
            Some(Duration::from_millis(diff as u64))
        } else {
            None
        }
    }

}

/// normalize addresss as ip:port
pub fn normalize_addr(addr: &mut String, default_port: &str) -> Result<()> {
    let mut parts = addr.split(':');
    let ip = parts.next().ok_or_else(||anyhow!("empty addr"))?;
    let r = parts.next(); 
    match r {
        Some(port) => { 
            if parts.next().is_some() {
                bail!("too many \":\" in addrs")
            }

            if ip.is_empty() {
                if port.is_empty() {
                    // addr = ":"    
                    *addr = format!("0.0.0.0:{}", default_port);    
                } else {
                    // addr = ":0"
                    *addr = format!("0.0.0.0:{}", port);
                }   
            }
        },
        None => {
            // addr = "0.0.0.0"
            addr.push(':');
            addr.push_str(default_port);
        }
    }
    Ok(())
}


#[derive(Debug)]
pub struct Latency {
    min: i64,
    max: i64,
    sum: i64,
    num: i64,
}

impl Latency {
    pub fn new() -> Self {
        Self {
            min: i64::MAX,
            max: i64::MIN,
            sum: 0, 
            num: 0,
        }
    }


    pub fn min(&self) -> i64 {
        self.min
    }

    pub fn max(&self) -> i64 {
        self.max
    }

    pub fn num(&self) -> i64 {
        self.num
    }

    pub fn sum(&self) -> i64 {
        self.sum
    }

    pub fn average(&self) -> i64 {
        if self.num > 0 {
            (self.sum + self.sum-1) / self.num
        } else {
            0
        }
    }

    pub fn observe(&mut self, latency: i64) {
        if latency < self.min {
            self.min = latency;
        }
        if latency > self.max {
            self.max = latency;
        }

        self.sum += latency;
        self.num += 1;
    }

    pub fn merge(&mut self, other: &Self) {
        if other.min < self.min {
            self.min = other.min;
        }

        if other.max > self.max {
            self.max = other.max;
        }

        self.sum += other.sum;
        self.num += other.num;
    }

    pub fn merge_iter<'a, I>(&mut self, mut iter: I) 
    where 
        I: Iterator<Item = &'a Self>
    {
        while let Some(other) = iter.next() {
            self.merge(other);
        }
    }

    pub fn from_iter<'a, I>(iter: I) -> Self
    where 
        I: Iterator<Item = &'a Self>
    {
        let mut self0 = Self::new();
        self0.merge_iter(iter);
        self0
    }
}

