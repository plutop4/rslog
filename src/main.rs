extern crate clap;
extern crate log;
extern crate redis;
extern crate stderrlog;
use std::thread::sleep;
use std::time::Duration;
mod slowlog;
use slowlog::SlowlogRecord;
mod slowlog_reader;
use slowlog_reader::SlowlogReader;
use std::convert::TryFrom;

struct Config {
    hostname: String,
    port: u16,
    interval: u64,
    verbosity: usize,
    quiet: bool,
}

fn get_cli_args() -> Result<Config, clap::Error> {
    let args = clap::App::from(clap::load_yaml!("cli.yaml"))
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .get_matches();
    let config = Config {
        hostname: args.value_of("hostname").unwrap().to_owned(),
        port: args.value_of("port").unwrap().parse().map_err(|_| {
            clap::Error::with_description(
                "Port mast be a in range 0-65535".to_owned(),
                clap::ErrorKind::ValueValidation,
            )
        })?,
        interval: args.value_of("interval").unwrap().parse().map_err(|_| {
            clap::Error::with_description(
                "Interval must be an integer".to_owned(),
                clap::ErrorKind::ValueValidation,
            )
        })?,
        verbosity: args.occurrences_of("verbosity") as usize,
        quiet: args.is_present("quiet"),
    };
    Ok(config)
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

fn main() {
    let config = get_cli_args().map_err(|e| e.exit()).unwrap();
    stderrlog::new()
        .timestamp(stderrlog::Timestamp::Second)
        .verbosity(config.verbosity)
        .quiet(config.quiet)
        .init()
        .unwrap();
    let redis_client = redis::Client::open((config.hostname, config.port)).unwrap();
    let mut sl_reader = create_slowlog_reader(redis_client, config.interval);

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
        sleep(Duration::new(config.interval, 0));
    }
}
