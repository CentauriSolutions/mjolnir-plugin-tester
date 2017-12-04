extern crate base64;
extern crate clap;
#[macro_use]
extern crate mjolnir_api;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

use std::env::home_dir;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, exit};

use base64::encode;
use clap::{App, Arg};

use mjolnir_api::{Alert, PluginEntry, Remediation, RemediationResult};

fn main() {
    let matches = App::new("mjolnir-tester")
        .arg(
            Arg::with_name("config")
                .long("config")
                .short("c")
                .help("Path to the config file to use")
                .takes_value(true)
                .required(true)
        )
        .arg(
            Arg::with_name("quick")
                .long("quick")
                .short("q")
                .help("Only validate that the plugin can register")
        )
        .get_matches();

    let config_path = matches.value_of("config").unwrap();
    let mut f = match File::open(&config_path) {
        Ok(f) => f,
        Err(e) => {
            println!("Couldn't open config file at {}: {:?}",config_path, e);
            exit(1);
        }
    };
    let mut config_string = String::new();
    match f.read_to_string(&mut config_string) {
        Ok(_) => {},
        Err(e) => {
            println!("Couldn't read config file at {}: {:?}",config_path, e);
            exit(1);
        }
    };
    let config: Plugin = match toml::from_str::<Config>(&config_string) {
        Ok(a) => a.plugin,
        Err(e) => {
            println!("Couldn't parse your config: {:?}", e);
            exit(1);
        }
    };
    println!("Have plugin: {:?}", config);
    let path = if config.path.contains("$HOME") || config.path.contains("~") {
        let mut home = match  home_dir() {
            Some(h) => h,
            None => {
                println!("Couldn't figure out where your HOME is");
                exit(1);
            }
        };

        // PathBuf::from( )
        let p = config.path.clone().replace("$HOME/", "").replace("~/", "");
        home.push(p);
        home
    } else {
        PathBuf::from(&config.path)
    };

    let plugin = match Command::new(&path).output() {
        Ok(output) => {
            match PluginEntry::try_from(
                &output.stdout,
                &path,
                ) {
                    Ok(plugin) => plugin,
                    Err(e) => {
                        println!("Invalid plugin registration for {}: {:?}", config.path, e);
                        exit(1);
                }
            }
        }
        Err(e) => {
            println!("Couldn't execute plugin: {:?}", e);
            println!("Tried executing: {}", path.display());
            exit(1);
        }
    };
    println!("Plugin registers as {:?}", plugin);
    if matches.is_present("quick") {
        exit(0);
    }
    let mut cmd = Command::new(&plugin.path);
    cmd.arg(format!("plugin={}", plugin.name));
    // cmd.arg(format!("body={}", body));
    for arg in &config.args {
        // info!("Adding {} to {:?}", arg, cmd);
        cmd.arg(&arg);
    }
    let remediation = Remediation {
            plugin: "Test".into(),
            target: None,
            args: vec![],
            alert: Some(Alert::new("test")
                .with_name("Mjolnir")
                .with_arg("This is a test Mjolnir alert".into())),
        };

    cmd.arg(format!("remediation={}", encode(&remediation.write_to_bytes().unwrap())));
    println!("Command is: {:?}", cmd);
    let remediation_result = match cmd.output() {
        Ok(output) => {
            match String::from_utf8(output.stdout) {
                Ok(s) => {
                    RemediationResult::from_string(&s)
                },
                Err(e) => RemediationResult::new().err(format!("{:?}", e)),
            }
        }
        Err(e) => RemediationResult::new().err(format!("{:?}", e))
    };
    // println!("{} returned the result {:?}", config.path, remediation_result);
    if remediation_result.result.is_err()  {
        exit(1);
    }
}

#[derive(Clone, Debug, Deserialize)]
struct Config {
    plugin: Plugin,
}

#[derive(Clone, Debug, Deserialize)]
struct Plugin {
    path: String,
    args: Vec<String>
}