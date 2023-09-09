use std::{
    io::Write,
    net::{SocketAddr, SocketAddrV4, UdpSocket, Ipv4Addr},
    time::{Instant, Duration},
    path::PathBuf, fs::OpenOptions,
};

use clap::{Parser, Subcommand};
use libc::time;
use rand::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct PollRequest {
    cookie: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct PollResponse {
    send_time: u64,
    cookie: u32,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Invocation {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Server {
        /// Port to listen on.
        #[clap(short,long,default_value="0.0.0.0:10456")]
        address: SocketAddr,
    },
    Client {
        /// Address of the server to connect to. (Note: the server is probably
        /// listening on port 10456.)
        #[clap(short,long)]
        address: SocketAddr,
        /// Logfile to dump CSV to. At the default poll interval of 5 seconds,
        /// will grow at a rate of approximately 600KB per day.
        #[clap(short,long)]
        logfile: PathBuf,
        /// Approximate polling interval in seconds. Polling more often than
        /// once per second isn't very useful.
        #[clap(short,long,default_value_t=5)]
        poll_interval: u64,
    },
}

fn get_send_time() -> u64 {
    let mut nau = 0;
    (unsafe { time(&mut nau) } as u64)
}

fn client(address: SocketAddr, logfile: PathBuf, poll_interval: u64) {
    let mut logfile = OpenOptions::new().create(true).append(true).open(logfile).unwrap();
    let sock = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))
        .expect("Unable to bind socket");
    sock.connect(&address)
        .expect("Unable to connect socket. (This shouldn't happen.)");
    println!("Client running. Control-C to quit.");
    let poll_interval = Duration::from_secs(poll_interval);
    let start_time = Instant::now();
    let mut next_poll = start_time;
    let mut last_cookie = None;
    let mut last_send_time = 0;
    let mut last_sent_at = start_time;
    let mut thread_rng = thread_rng();
    let mut buf = [0; 1536];
    let mut hours_running = 0;
    let mut samples_collected: u64 = 0;
    loop {
        let now = Instant::now();
        let new_hours_running = (now - start_time).as_secs() / 3600;
        if new_hours_running != hours_running {
            hours_running = new_hours_running;
            println!("Been running for {hours_running} hour{s} and collected {samples_collected} samples.",
            s = if hours_running == 1 { "" } else { "s" });
        }
        if next_poll <= now {
            let send_time = get_send_time();
            let cookie = thread_rng.gen();
            let request = PollRequest {
                cookie: cookie,
            };
            last_cookie = Some(cookie);
            let request = serde_json::to_string(&request).unwrap();
            let _ = sock.send(&request.as_bytes());
            last_sent_at = now;
            last_send_time = send_time;
            next_poll += poll_interval;
            if next_poll <= now {
                // you're too slow! (probably went to sleep)
                next_poll = now + poll_interval;
            }
        }
        assert!(next_poll > now);
        let _ = sock.set_read_timeout(Some(next_poll - now));
        if let Ok(len) = sock.recv(&mut buf[..]) {
            if let Ok(response) = serde_json::from_slice::<PollResponse>(&buf[..len]) {
                if last_cookie != Some(response.cookie) {
                    println!("Received a bad cookie...!");
                } else {
                    last_cookie = None;
                    let buf = format!("{last_send_time},{time_differential},{delay}\n",
                    time_differential = (response.send_time as i64) - (last_send_time) as i64,
                        delay = (Instant::now() - last_sent_at).as_secs_f32());
                    logfile.write_all(buf.as_bytes()).expect("Error appending to log file");
                    samples_collected += 1;
                }
            }
        }
    }
}

fn server(listen_address: SocketAddr) {
    let sock = UdpSocket::bind(&listen_address)
        .expect("Unable to bind socket");
    println!("Server running on {listen_address:?}. Control-C to quit.");
    let mut buf = [0; 1536];
    loop {
        let (len, addr) = sock.recv_from(&mut buf[..]).unwrap();
        if let Ok(poll) = serde_json::from_slice::<PollRequest>(&buf[..len]) {
            let send_time = get_send_time();
            let response = PollResponse {
                send_time,
                cookie: poll.cookie,
            };
            println!("sending response {response:?} to {addr:?}");
            let response = serde_json::to_string(&response).unwrap();
            let _ = sock.send_to(response.as_bytes(), addr);
        }
    }
}

fn main() {
    let invocation = Invocation::parse();
    match invocation.command {
        Command::Client { address, logfile, poll_interval } => {
            client(address, logfile, poll_interval)
        },
        Command::Server { address } => {
            server(address)
        },
    }
}
