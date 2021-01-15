use std::thread::sleep;
use std::time::Duration;
mod slowlog;
use slowlog::SlowlogRecord;
mod argument_parsing;
mod slowlog_reader;
use argument_parsing::OutputFormat;
use slowlog_reader::SlowlogReader;
use std::convert::TryFrom;

#[derive(Clone)]
pub struct ConnectionProvider {
    client: redis::Client,
    timeout: u64,
}

impl From<(redis::Client, u64)> for ConnectionProvider {
    fn from(arg: (redis::Client, u64)) -> ConnectionProvider {
        ConnectionProvider {
            client: arg.0,
            timeout: arg.1,
        }
    }
}

impl ConnectionProvider {
    pub fn get_connection(&self) -> redis::RedisResult<redis::Connection> {
        self.client
            .get_connection_with_timeout(Duration::from_secs(self.timeout))
    }
}

fn print_rec(r: &SlowlogRecord, format: &OutputFormat) {
    match format {
        OutputFormat::Text => {
            println!(
                "[{}] id: {},\tduration: {},\tclient: {},\tclient_name: {},\tcommand: {:?}",
                r.time, r.id, r.duration, r.client_socket, r.client_name, r.command
            )
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(r).unwrap())
        }
    }
}

fn error_handler(e: redis::RedisError) {
    match e.kind() {
        redis::ErrorKind::IoError => {
            log::error!("Can't establish connection to redis cluster: {}", e)
        }
        _ => unimplemented!("Error not handled: {}({:?})", e, e.kind()),
    }
}

fn create_slowlog_reader(con_provider: ConnectionProvider, interval: u64) -> SlowlogReader {
    log::debug!("Creating slowlog reader");
    loop {
        match SlowlogReader::try_from(con_provider.clone()) {
            Err(e) => error_handler(e),
            Ok(slr) => return slr,
        }
        sleep(Duration::new(interval, 0))
    }
}

fn read_once(con_provider: ConnectionProvider, config: &argument_parsing::Config) {
    match {
        move || -> Result<(), redis::RedisError> {
            for r in slowlog_reader::get_slowlog(&mut con_provider.get_connection()?, 128)?.iter() {
                print_rec(r, &config.output_format)
            }
            Ok(())
        }
    }() {
        Err(e) => error_handler(e),
        Ok(_) => std::process::exit(0),
    }
}

fn read_continiously(con_provider: ConnectionProvider, config: &argument_parsing::Config) {
    let mut sl_reader = create_slowlog_reader(con_provider, config.interval);

    loop {
        match sl_reader
            .get()
            .map_err(|e| sl_reader.redis_error_handler(e))
        {
            Ok(records) => {
                for r in records.iter().rev() {
                    print_rec(r, &config.output_format)
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

pub fn main() {
    let config = argument_parsing::get_config()
        .map_err(|e| e.exit())
        .unwrap();
    stderrlog::new()
        .timestamp(stderrlog::Timestamp::Second)
        .verbosity(config.verbosity)
        .quiet(config.quiet)
        .init()
        .unwrap();
    let redis_client = redis::Client::open((&config.hostname, config.port)).unwrap();
    let connection_provider = ConnectionProvider::from((redis_client, config.interval));
    if config.follow {
        read_continiously(connection_provider, &config)
    } else {
        read_once(connection_provider, &config)
    }
}
