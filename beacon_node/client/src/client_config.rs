use clap::ArgMatches;
use fork_choice::ForkChoiceAlgorithm;
use network::NetworkConfig;
use slog::{error, info, o, Drain};
use std::fs;
use std::fs::OpenOptions;
use std::net::SocketAddr;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::sync::Mutex;
use types::multiaddr::Protocol;
use types::multiaddr::ToMultiaddr;
use types::Multiaddr;
use types::{ChainSpec, EthSpec, LighthouseTestnetEthSpec};

#[derive(Debug, Clone)]
pub enum DBType {
    Memory,
    Disk,
}

/// Stores the client configuration for this Lighthouse instance.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub data_dir: PathBuf,
    pub log_file: String,
    pub spec: ChainSpec,
    pub net_conf: network::NetworkConfig,
    pub fork_choice: ForkChoiceAlgorithm,
    pub db_type: DBType,
    pub db_name: PathBuf,
    pub rpc_conf: rpc::RPCConfig,
    //pub ipc_conf:
}

impl Default for ClientConfig {
    /// Build a new lighthouse configuration from defaults.
    fn default() -> Self {
        let data_dir = {
            let home = dirs::home_dir().expect("Unable to determine home dir.");
            home.join(".lighthouse/")
        };
        fs::create_dir_all(&data_dir)
            .unwrap_or_else(|_| panic!("Unable to create {:?}", &data_dir));

        let default_spec = LighthouseTestnetEthSpec::spec();
        let default_net_conf = NetworkConfig::new(default_spec.boot_nodes.clone());

        Self {
            data_dir: data_dir.clone(),
            log_file: "".to_string(),
            // default to foundation for chain specs
            spec: default_spec,
            net_conf: default_net_conf,
            // default to bitwise LMD Ghost
            fork_choice: ForkChoiceAlgorithm::BitwiseLMDGhost,
            // default to memory db for now
            db_type: DBType::Memory,
            // default db name for disk-based dbs
            db_name: data_dir.join("chain_db"),
            rpc_conf: rpc::RPCConfig::default(),
        }
    }
}

impl ClientConfig {
    /// Parses the CLI arguments into a `Config` struct.
    pub fn parse_args(args: ArgMatches, log: &mut slog::Logger) -> Result<Self, &'static str> {
        let mut config = ClientConfig::default();

        /* Network related arguments */

        // Custom p2p listen port
        if let Some(port_str) = args.value_of("port") {
            if let Ok(port) = port_str.parse::<u16>() {
                config.net_conf.listen_port = port;
                // update the listening multiaddrs
                for address in &mut config.net_conf.listen_addresses {
                    address.pop();
                    address.append(Protocol::Tcp(port));
                }
            } else {
                error!(*log, "Invalid port"; "port" => port_str);
                return Err("Invalid port");
            }
        }
        // Custom listening address ipv4/ipv6
        // TODO: Handle list of addresses
        if let Some(listen_address_str) = args.value_of("listen-address") {
            if let Ok(listen_address) = listen_address_str.parse::<IpAddr>() {
                let multiaddr = SocketAddr::new(listen_address, config.net_conf.listen_port)
                    .to_multiaddr()
                    .expect("Invalid listen address format");
                config.net_conf.listen_addresses = vec![multiaddr];
            } else {
                error!(*log, "Invalid IP Address"; "Address" => listen_address_str);
                return Err("Invalid IP Address");
            }
        }

        // Custom bootnodes
        // TODO: Handle list of addresses
        if let Some(boot_addresses_str) = args.value_of("boot-nodes") {
            if let Ok(boot_address) = boot_addresses_str.parse::<Multiaddr>() {
                config.net_conf.boot_nodes.append(&mut vec![boot_address]);
            } else {
                error!(*log, "Invalid Bootnode multiaddress"; "Multiaddr" => boot_addresses_str);
                return Err("Invalid IP Address");
            }
        }

        /* Filesystem related arguments */

        // Custom datadir
        if let Some(dir) = args.value_of("datadir") {
            config.data_dir = PathBuf::from(dir.to_string());
        };

        /* RPC related arguments */

        if args.is_present("rpc") {
            config.rpc_conf.enabled = true;
        }

        if let Some(rpc_address) = args.value_of("rpc-address") {
            if let Ok(listen_address) = rpc_address.parse::<Ipv4Addr>() {
                config.rpc_conf.listen_address = listen_address;
            } else {
                error!(*log, "Invalid RPC listen address"; "Address" => rpc_address);
                return Err("Invalid RPC listen address");
            }
        }

        if let Some(rpc_port) = args.value_of("rpc-port") {
            if let Ok(port) = rpc_port.parse::<u16>() {
                config.rpc_conf.port = port;
            } else {
                error!(*log, "Invalid RPC port"; "port" => rpc_port);
                return Err("Invalid RPC port");
            }
        }

        match args.value_of("db") {
            Some("disk") => config.db_type = DBType::Disk,
            Some("memory") => config.db_type = DBType::Memory,
            _ => unreachable!(), // clap prevents this.
        };

        /* Logging */

        if let Some(logfile) = args.value_of("logfile") {
            config.log_file = logfile.to_string();
            // Panics if bad file
            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&config.log_file)
                .unwrap();

            info!(*log, "Log file specified output will now be written to {} in json", &config.log_file);
            let drain = Mutex::new(slog_json::Json::default(file)).fuse();
            let drain = slog_async::Async::new(drain).build().fuse();
            *log = slog::Logger::root(drain, o!());
        };

        Ok(config)
    }
}
