# Rslog
A small utility for viewing and monitoring redis slowlog.

# Build
You will need rustc and cargo to build it.
Once you have it run the following command:
```
cargo build --release
```
The resulting binary will be located in ./target/release/rslog.

# Usage

```
$ rslog --help
Redis slowlog reader 0.1.0
Rostyslav Ivanika <Rostyslav.Ivanika@gmail.com>
Prints redis slowlog to stdout

USAGE:
    rslog [FLAGS] [OPTIONS]

FLAGS:
    -f, --follow     Checks for new records in slowlog and prints if any
        --help       Prints help information
        --json       Format output as newline separated JSON
    -q, --quiet      Silence all error messages
    -v               Sets the level of verbosity
    -V, --version    Prints version information

OPTIONS:
    -h, --hostname <hostname>    Server hostname [default: 127.0.0.1]
    -i, --interval <interval>    Seconds between trying to get new messages from slowlog [default:
                                 5]
    -p, --port <port>            Server port [default: 6379]
        --timeout <timeout>      Timout for redis connection [default: 30]
```


