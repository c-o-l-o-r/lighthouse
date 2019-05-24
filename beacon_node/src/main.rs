extern crate slog;
extern crate slog_json;

mod run;

use clap::{App, Arg};
use client::ClientConfig;
use slog::{error, o, Drain};


fn main() {
    // Logging
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let mut log = slog::Logger::root(drain, o!());

    // CLI
    let matches = App::new("Lighthouse")
        .version(version::version().as_str())
        .author("Sigma Prime <contact@sigmaprime.io>")
        .about("Eth 2.0 Client")
        // file system related arguments
        .arg(
            Arg::with_name("datadir")
                .long("datadir")
                .value_name("DIR")
                .help("Data directory for keys and databases.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("logfile")
                .long("logfile")
                .value_name("logfile")
                .help("File path where output will be written.")
                .takes_value(true),
        )
        // network related arguments
        .arg(
            Arg::with_name("listen-address")
                .long("listen-address")
                .value_name("Listen Address")
                .help("The Network address to listen for p2p connections.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .long("port")
                .value_name("PORT")
                .help("Network listen port for p2p connections.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("boot-nodes")
                .long("boot-nodes")
                .value_name("BOOTNODES")
                .help("A list of comma separated multi addresses representing bootnodes to connect to.")
                .takes_value(true),
        )
        // rpc related arguments
        .arg(
            Arg::with_name("rpc")
                .long("rpc")
                .value_name("RPC")
                .help("Enable the RPC server.")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("rpc-address")
                .long("rpc-address")
                .value_name("RPCADDRESS")
                .help("Listen address for RPC endpoint.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("rpc-port")
                .long("rpc-port")
                .value_name("RPCPORT")
                .help("Listen port for RPC endpoint.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("db")
                .long("db")
                .value_name("DB")
                .help("Type of database to use.")
                .takes_value(true)
                .possible_values(&["disk", "memory"])
                .default_value("memory"),
        )
        .get_matches();

    // invalid arguments, panic
    let config = ClientConfig::parse_args(matches, &mut log).unwrap();

    match run::run_beacon_node(config, &log) {
        Ok(_) => {}
        Err(e) => error!(log, "Beacon node failed because {:?}", e),
    }
}
