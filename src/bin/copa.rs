#[macro_use]
extern crate clap;
use clap::Arg;

#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;
use slog::Drain;

extern crate ccp_copa;
use ccp_copa::CopaConfig;

extern crate portus;

arg_enum! {
#[derive(Clone)]
enum DeltaModeArg {
    NoTCP,
    Auto,
}
}

impl Into<ccp_copa::DeltaModeConf> for DeltaModeArg {
    fn into(self) -> ccp_copa::DeltaModeConf {
        match self {
            DeltaModeArg::NoTCP => ccp_copa::DeltaModeConf::NoTCP,
            DeltaModeArg::Auto => ccp_copa::DeltaModeConf::Auto,
        }
    }
}

fn make_logger() -> slog::Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    slog::Logger::root(drain, o!())
}

fn make_args(log: slog::Logger) -> Result<(CopaConfig, String), std::num::ParseIntError> {
    let matches = clap::App::new("CCP Copa")
        .version("0.1.0")
        .author("Venkat Arun <venkatar@mit.edu>")
        .about("Implementation of Copa Congestion Control")
        .arg(Arg::with_name("ipc")
             .long("ipc")
             .help("Sets the type of ipc to use: (netlink|unix)")
             .default_value("unix")
             .validator(portus::algs::ipc_valid))
        .arg(Arg::with_name("init_cwnd")
             .long("init_cwnd")
             .help("Sets the initial congestion window, in bytes. Setting 0 will use datapath default.")
             .default_value("0"))
        .arg(Arg::with_name("default_delta")
             .long("default_delta")
             .help("Delta to use when in default mode.")
             .default_value("0.5"))
        .arg(Arg::with_name("delta_mode")
             .long("delta_mode")
             .help("Delta mode to use. NoTcp for guaranteed no cross traffic TCP flows.")
             .possible_values(&DeltaModeArg::variants())
             .default_value("Auto"))
        .get_matches();

    Ok((
        ccp_copa::CopaConfig {
            logger: Some(log),
            init_cwnd: u32::from_str_radix(matches.value_of("init_cwnd").unwrap(), 10)?,
            default_delta: (matches.value_of("default_delta").unwrap())
                .parse()
                .unwrap(),
            delta_mode: value_t!(matches, "delta_mode", DeltaModeArg)
                .unwrap()
                .into(),
        },
        String::from(matches.value_of("ipc").unwrap()),
    ))
}

fn main() {
    let log = make_logger();
    let (cfg, ipc) = make_args(log.clone())
        .map_err(|e| warn!(log, "bad argument"; "err" => ?e))
        .unwrap();

    info!(log, "configured Copa";
          "ipc" => ipc.clone(),
          "init_cwnd" => cfg.init_cwnd,
          "default_delta" => cfg.default_delta,
          "delta_mode" => ?cfg.delta_mode,
    );

    //portus::start!(ipc.as_str(), Some(log), cfg, portus::ipc::Blocking, 1).unwrap()
    portus::start!(ipc.as_str(), Some(log), cfg).unwrap()
}
