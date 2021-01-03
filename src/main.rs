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

fn create_slowlog_reader(client: redis::Client) -> SlowlogReader {
    log::debug!("Creating slowlog reader");
    loop {
        match SlowlogReader::try_from(client.clone()) {
            Err(e) => error_handler(e),
            Ok(slr) => return slr,
        }
        sleep(Duration::new(2, 0))
    }
}

fn main() {
    // Parse args
    let args = clap::App::from(clap::load_yaml!("cli.yaml"))
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .get_matches();
    let hostname = args.value_of("hostname").unwrap();
    let port: u16 = args.value_of("port").unwrap().parse().unwrap();
    let verbosity = args.occurrences_of("verbose") as usize;
    let quiet = args.is_present("quiet");
    // Init logger
    stderrlog::new()
        .timestamp(stderrlog::Timestamp::Second)
        .verbosity(verbosity)
        .quiet(quiet)
        .init()
        .unwrap();
    let redis_client = redis::Client::open((hostname, port)).unwrap();
    let mut sl_reader = create_slowlog_reader(redis_client);

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
        sleep(Duration::new(2, 0));
    }
}
