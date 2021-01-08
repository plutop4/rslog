use clap::{App, Arg};

pub struct Config {
    pub hostname: String,
    pub port: u16,
    pub follow: bool,
    pub interval: u64,
    pub verbosity: usize,
    pub quiet: bool,
}

pub fn get_config() -> Result<Config, clap::Error> {
    let args = App::new("Redis slowlog reader")
        .about("Prints redis slowlog to stdout")
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .arg(
            Arg::new("hostname")
                .about("Server hostname")
                .short('h')
                .takes_value(true)
                .default_value("127.0.0.1"),
        )
        .arg(
            Arg::new("port")
                .about("Server port")
                .short('p')
                .takes_value(true)
                .default_value("6379")
                .validator(|port| {
                    if port.parse::<u16>().is_err() {
                        Err("Port mast be a in range 0-65535")
                    } else {
                        Ok(())
                    }
                }),
        )
        .arg(
            Arg::from("-f --follow 'checks for new records in slowlog and prints if any'")
                .takes_value(false),
        )
        .arg(
            Arg::new("interval")
                .about("seconds between trying to get new messages from slowlog")
                .short('i')
                .default_value("5")
                .validator(|interval| {
                    if interval.parse::<u64>().is_err() {
                        Err("Interval must be an integer")
                    } else {
                        Ok(())
                    }
                }),
        )
        .arg(
            Arg::new("verbosity")
                .about("Sets the level of verbosity")
                .short('v')
                .multiple(true)
                .takes_value(false),
        )
        .arg(
            Arg::new("quiet")
                .about("Silence all error messages")
                .short('q')
                .takes_value(false),
        )
        .get_matches();

    let config = Config {
        hostname: args.value_of("hostname").unwrap().to_owned(),
        port: args.value_of("port").unwrap().parse().unwrap(),
        interval: args.value_of("interval").unwrap().parse().unwrap(),
        follow: args.is_present("follow") || args.occurrences_of("interval") > 0,
        verbosity: args.occurrences_of("verbosity") as usize,
        quiet: args.is_present("quiet"),
    };
    Ok(config)
}
