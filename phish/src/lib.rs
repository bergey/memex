#[macro_use]
extern crate lazy_static;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use clap::Parser;
use log::{debug, info};
use prometheus::{self, Histogram, IntCounter};
use rand_distr::{Distribution, Exp};
use tokio::spawn;
use tokio::time::{Duration, Instant, Sleep, sleep, sleep_until};

#[derive(Parser, Debug, Clone)]
struct Cli {
    #[arg(long, short)]
    /// mean think time in seconds (exponentially distributed)
    think_time_s: Option<f64>,

    #[arg(long, short = 'u', default_value_t = 1)]
    num_users: u64,

    #[arg(long)]
    /// stop after sending (approximately) this many messages
    stop_after: Option<u64>,

    #[arg(long)]
    /// time in seconds to ramp up to full number of users
    ramp_time: Option<f64>,

    #[arg(long, default_value_t)]
    /// time in seconds to wait after sending messages before exit
    sleep: f64,
}

lazy_static! {
    static ref ROUND_TRIP_TIME: Histogram = prometheus::register_histogram!(
        "round_trip_time",
        "client-side time to persist one new line to server"
    )
    .unwrap();
    static ref RECEIVED_COUNT: IntCounter =
        prometheus::register_int_counter!("received_count", "number of ws messages received").unwrap();
    // undecided whether to use this in place of the raw Atomic64 counter
    static ref SENT_COUNT: IntCounter =
        prometheus::register_int_counter!("sent_count", "number of lines sent to server").unwrap();
}

// connect is called once per user
// act is called repeatedly for each user, with randomized sleep n
// one T per user, can be used to retain state between calls to act
pub async fn phish_main<T, C, A, CF, AF>(connect: C, act: A) -> anyhow::Result<()>
where
    T: Send + Sync +'static,
    C: Fn(usize) -> CF,
    CF: Future<Output = T> + Send + 'static,
    A: Fn(T, Timer) -> AF + Clone + Send + Sync + 'static,
    AF: Future<Output = ()> + Send + Sync + 'static,
{
    let cli = Cli::parse();
    env_logger::init();

    match cli.ramp_time {
        Some(ramp_s) => info!(
            "opening {} connections at rate {} per second",
            cli.num_users,
            cli.num_users as f64 / ramp_s
        ),
        None => info!(
            "opening {} connections as quickly as possible",
            cli.num_users
        ),
    }

    let mut joins = Vec::new();
    let mut time = Instant::now();
    for i in 1..=cli.num_users as usize {
        let t = spawn(connect(i));
        joins.push(t);
        debug!("started {i} connection attempts");
        if let Some(ramp_s) = cli.ramp_time {
            time += Duration::from_secs_f64(ramp_s / cli.num_users as f64);
            sleep_until(time).await;
        }
    }

    let mut ts = Vec::new();
    for j in joins {
        ts.push(j.await?);
        if ts.len() as u64 % (std::cmp::max(1, cli.num_users / 10)) == 0
            || ts.len() as u64 == cli.num_users
        {
            // approximate, we may have opened more if they complete out of order
            info!("opened {} connections", ts.len());
        } else {
            debug!("opened {} connections", ts.len());
        }
    }

    info!("starting user behavior");
    let mut joins = Vec::new();
    for t in ts.into_iter() {
        // TODO share atomic counter
        let timer = Timer::new(&cli);
        joins.push(spawn(act(t, timer)))
    }
    for j in joins {
        let _ = j.await;
    }

    sleep(Duration::from_secs_f64(cli.sleep)).await;

    Ok(())
}

#[derive(Clone)]
pub struct Timer {
    deadline: Instant,
    think: Option<Exp<f64>>,
    stop_after: Option<u64>,
    count: Arc<AtomicU64>,
}

impl Timer {
    fn new(cli: &Cli) -> Self {
        Timer {
            deadline: Instant::now(),
            think: cli.think_time_s.map(|λ| Exp::new(λ).unwrap()),
            stop_after: cli.stop_after,
            count: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl Iterator for Timer {
    type Item = Sleep;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.count.fetch_add(1, Ordering::Relaxed);
        if let Some(n) = self.stop_after {
            if i == n - 1 {
                return None
            }
        }
        match self.think {
            Some(think) => {
                self.deadline += Duration::from_secs_f64(think.sample(&mut rand::rng()));
                Some(sleep_until(self.deadline))
            }
            None => Some(sleep(Duration::ZERO))
        }
    }
}
