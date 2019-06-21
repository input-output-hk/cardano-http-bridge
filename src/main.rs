#[macro_use]
extern crate clap;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
#[macro_use]
extern crate log;
extern crate env_logger;

extern crate iron;
extern crate router;

extern crate cardano;
extern crate cardano_storage;
extern crate exe_common;

use std::path::{
    PathBuf,
    Path,
};

mod config;
mod handlers;
mod service;

use self::config::{hermes_path, Config};
use exe_common::config::net;

fn main() {
    use clap::{App, Arg, SubCommand};

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .subcommand(
            SubCommand::with_name("start")
                .about("start explorer server")
                .arg(
                    Arg::with_name("PORT NUMBER")
                        .long("port")
                        .takes_value(true)
                        .value_name("PORT NUMBER")
                        .help("set the port number to listen to")
                        .required(false)
                        .default_value("80"),
                )
                .arg(
                    Arg::with_name("NETWORKS DIRECTORY")
                        .long("networks-dir")
                        .takes_value(true)
                        .value_name("NETWORKS DIRECTORY")
                        .help("the relative or absolute directory of the networks to server")
                        .required(false),
                )
                .arg(
                    Arg::with_name("TEMPLATE")
                        .long("template")
                        .takes_value(true)
                        .value_name("TEMPLATE")
                        .help("either 'mainnet' or 'testnet'; may be given multiple times")
                        .required(false)
                        .multiple(true)
                        .default_value("mainnet")
                )
                .arg(
                    Arg::with_name("no-sync")
                        .long("no-sync")
                        .help("disable synchronizing with the upstream network"),
                )
                .arg(
                    Arg::with_name("verbose")
                        .long("verbose")
                        .help("display debugging information in the log output"),
                )
                .arg(
                    Arg::with_name("quiet")
                        .long("quiet")
                        .help("suppress all log output apart from errors"),
                )
                .arg(
                    Arg::with_name("silent")
                        .long("silent")
                        .help("suppress all log output"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        ("start", Some(args)) => {

            // Determine the verbosity of logging:
            let arg_verbose = args.is_present("verbose");
            let arg_quiet   = args.is_present("quiet");
            let arg_silent  = args.is_present("silent");

            let log_filter_level = match (arg_verbose, arg_quiet, arg_silent) {
                (false, false, false) => {log::LevelFilter::Info}  // Default
                (true , false, false) => {log::LevelFilter::Trace} // Verbose
                (false, true , false) => {log::LevelFilter::Error} // Quiet
                (false, false, true ) => {log::LevelFilter::Off}   // Silent
                _                     =>
                    {panic!("Error: At most one of the following arguments \
                             may be specified: --verbose --quiet --silent")}
            };

            env_logger::Builder::from_default_env()
                .filter_level(log_filter_level)
                .init();

            let mut cfg = Config::new(
                PathBuf::from(
                    value_t!(args.value_of("NETWORKS DIRECTORY"), String).unwrap_or(
                        hermes_path()
                            .unwrap()
                            .join("networks")
                            .to_str()
                            .unwrap()
                            .to_string(),
                    ),
                ),
                value_t!(args.value_of("PORT NUMBER"), u16).unwrap(),
            );

            ::std::fs::create_dir_all(cfg.root_dir.clone()).expect("create networks directory");
            info!("Created networks directory {:?}", cfg.root_dir);

            for template in args.values_of("TEMPLATE").unwrap() {
                let net_cfg = match template {
                    "mainnet" => net::Config::mainnet(),
                    "staging" => net::Config::staging(),
                    "testnet" => net::Config::testnet(),
                    filepath  => {
                        let path = Path::new(filepath);
                        match net::Config::from_file(path) {
                            None => panic!("unknown or missing template '{}'", template),
                            Some(cfg) => cfg,
                        }
                    }
                };

                cfg.add_network(template, &net_cfg).unwrap();
            }

            cfg.sync = !args.is_present("no-sync");

            info!("Starting {}-{}", crate_name!(), crate_version!());
            service::start(cfg);
        }
        _ => {
            println!("{}", matches.usage());
            ::std::process::exit(1);
        }
    }
}
