use clap::{App, Arg};

pub enum OutputFormat {
    Text,
    Json,
}

pub struct Config {
    pub hostname: String,
    pub port: u16,
    pub follow: bool,
    pub interval: u64,
    pub verbosity: usize,
    pub quiet: bool,
    pub timeout: u64,
    pub output_format: OutputFormat,
}

macro_rules! is_parsable {
    ($t: ty, $s: literal) => {
        |value| match value.parse::<$t>() {
            Err(_) => Err($s),
            Ok(_) => Ok(()),
        }
    };
}

pub fn get_config() -> Result<Config, clap::Error> {
    let args = App::new("Redis slowlog reader")
        .about("Prints redis slowlog to stdout")
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .arg(
            Arg::from("--hostname -h 'Server hostname'")
                .takes_value(true)
                .default_value("127.0.0.1"),
        )
        .arg(
            Arg::from("--port -p 'Server port'")
                .takes_value(true)
                .default_value("6379")
                .validator(is_parsable!(u16, "Port mast be a in range 0-65535")),
        )
        .arg(
            Arg::from("-f --follow 'Checks for new records in slowlog and prints if any'")
                .takes_value(false),
        )
        .arg(
            Arg::from("--interval -i 'Seconds between trying to get new messages from slowlog'")
                .default_value("5")
                .validator(is_parsable!(u64, "Interval must be an integer")),
        )
        .arg(
            Arg::new("verbosity")
                .about("Sets the level of verbosity")
                .short('v')
                .multiple(true)
                .takes_value(false),
        )
        .arg(Arg::from("--quiet -q 'Silence all error messages'").takes_value(false))
        .arg(
            Arg::from("--timeout 'Timout for redis connection'")
                .takes_value(true)
                .validator(is_parsable!(u64, "Timeout must be a positive integer"))
                .default_value("30"),
        )
        .arg(Arg::from("--json 'Format output as newline separated JSON'").takes_value(false))
        .get_matches();

    let config = Config {
        hostname: args.value_of("hostname").unwrap().to_owned(),
        port: args.value_of("port").unwrap().parse().unwrap(),
        interval: args.value_of("interval").unwrap().parse().unwrap(),
        follow: args.is_present("follow") || args.occurrences_of("interval") > 0,
        verbosity: args.occurrences_of("verbosity") as usize,
        quiet: args.is_present("quiet"),
        timeout: args.value_of("timeout").unwrap().parse().unwrap(),
        output_format: if args.is_present("json") {OutputFormat::Json} else {OutputFormat::Text},
    };
    Ok(config)
}
