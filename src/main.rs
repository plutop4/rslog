extern crate clap;
extern crate log;
extern crate redis;
extern crate stderrlog;
use std::thread::sleep;
use std::time::Duration;
mod slowlog;
use slowlog::SlowlogRecord;
mod argument_parsing;
mod slowlog_reader;
use slowlog_reader::SlowlogReader;
use std::convert::TryFrom;
#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref CONFIG: argument_parsing::Config = argument_parsing::get_config()
        .map_err(|e| e.exit())
        .unwrap();
}

fn print_rec(r: &SlowlogRecord) {
    println!(
        "[{}] id: {},\tduration: {},\tclient: {},\tclient_name: {},\tcommand: {:?}",
        r.time, r.id, r.duration, r.client_socket, r.client_name, r.command
    )
}

fn error_handler(e: redis::RedisError) {
    match e.kind() {
        redis::ErrorKind::IoError => {
            log::error!("Can't establish connection to redis cluster: {}", e)
        }
        _ => unimplemented!("Error not handled: {}({:?})", e, e.kind()),
    }
}

fn create_slowlog_reader(client: redis::Client, interval: u64) -> SlowlogReader {
    log::debug!("Creating slowlog reader");
    loop {
        match SlowlogReader::try_from(client.clone()) {
            Err(e) => error_handler(e),
            Ok(slr) => return slr,
        }
        sleep(Duration::new(interval, 0))
    }
}

fn read_once(client: redis::Client) {
    match {
        move || -> Result<(), redis::RedisError> {
            for r in slowlog_reader::get_slowlog(&mut client.get_connection()?, 128)?.iter() {
                print_rec(r)
            }
            Ok(())
        }
    }() {
        Err(e) => error_handler(e),
        Ok(_) => std::process::exit(0),
    }
}

fn read_continiously(client: redis::Client) {
    let mut sl_reader = create_slowlog_reader(client, CONFIG.interval);

    loop {
        match sl_reader
            .get()
            .map_err(|e| sl_reader.redis_error_handler(e))
        {
            Ok(records) => {
                for r in records.iter().rev() {
                    print_rec(r)
                }
            }
            Err(e) => {
                if let Err(e) = e {
                    error_handler(e)
                }
            }
        }
        sleep(Duration::new(CONFIG.interval, 0));
    }
}

fn main() {
    stderrlog::new()
        .timestamp(stderrlog::Timestamp::Second)
        .verbosity(CONFIG.verbosity)
        .quiet(CONFIG.quiet)
        .init()
        .unwrap();
    let redis_client = redis::Client::open((&CONFIG.hostname, CONFIG.port)).unwrap();
    if CONFIG.follow {
        read_continiously(redis_client)
    } else {
        read_once(redis_client)
    }
}
