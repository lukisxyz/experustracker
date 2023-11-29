use clap::{App, Arg};
use log::warn;
use serde::{Deserialize, Serialize};
use std::env;
use std::error;
use std::path::Path;

// configuration for server listener
#[derive(Debug, Deserialize, Serialize)]
pub struct ListenerConfig {
    pub host: String,
    pub port: u32,
    pub read_timeout: u32,
    pub write_timeout: u32,
    pub idle_timeout: u32,
}

impl ListenerConfig {
    fn default() -> Self {
        ListenerConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            read_timeout: 25,
            write_timeout: 25,
            idle_timeout: 300,
        }
    }
    fn load_from_env(&mut self) {
        if let Some(host) = load_env_str("LISTENER_HOST") {
            self.host = host;
        }
        if let Some(port) = load_env_uint("LISTENER_PORT") {
            self.port = port;
        }
        if let Some(read_timeout) = load_env_uint("LISTENER_READ_TIMEOUT") {
            self.read_timeout = read_timeout;
        }
        if let Some(write_timeout) = load_env_uint("LISTENER_WRITE_TIMEOUT") {
            self.write_timeout = write_timeout;
        }
        if let Some(idle_timeout) = load_env_uint("LISTENER_IDLE_TIMEOUT") {
            self.idle_timeout = idle_timeout;
        }
    }
}

// configuration for database
#[derive(Debug, Deserialize, Serialize)]
pub struct PgConfig {
    pub host: String,
    pub port: u32,
    pub db_name: String,
    pub ssl_mode: String,
    pub password: String,
    pub username: String,
}

impl PgConfig {
    fn default() -> Self {
        PgConfig {
            host: "127.0.0.1".to_string(),
            port: 5433,
            db_name: "postgres".to_string(),
            ssl_mode: "disable".to_string(),
            password: "password".to_string(),
            username: "postgres".to_string(),
        }
    }
    pub fn db_string(&self) -> String {
        let db_conn_string = format!(
            "postgresql://{}:{}@{}:{}/{}?sslmode={}",
            self.username, self.password, self.host, self.port, self.db_name, self.ssl_mode
        );
        db_conn_string
    }
    fn load_from_env(&mut self) {
        if let Some(host) = load_env_str("DB_HOST") {
            self.host = host;
        }
        if let Some(port) = load_env_uint("DB_PORT") {
            self.port = port;
        }
        if let Some(db_name) = load_env_str("DB_NAME") {
            self.db_name = db_name;
        }
        if let Some(ssl_mode) = load_env_str("DB_SSL") {
            self.ssl_mode = ssl_mode;
        }
        if let Some(password) = load_env_str("DB_PASSWORD") {
            self.password = password;
        }
        if let Some(username) = load_env_str("DB_USER") {
            self.username = username;
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub listen: ListenerConfig,
    pub db: PgConfig,
}

impl Config {
    fn default() -> Self {
        let default_pg_config = PgConfig::default();
        let default_listener_config = ListenerConfig::default();
        Config {
            listen: default_listener_config,
            db: default_pg_config,
        }
    }

    fn load_from_env(&mut self) {
        self.listen.load_from_env();
        self.db.load_from_env();
    }
}

fn load_env_str(key: &str) -> Option<String> {
    env::var(key).ok()
}

fn load_env_uint(key: &str) -> Option<u32> {
    env::var(key).ok().and_then(|s| s.parse().ok())
}

fn load_cfg_from_file(file_path: &str) -> Result<Config, Box<dyn error::Error>> {
    if Path::new(file_path).exists() {
        let content = std::fs::read_to_string(file_path)?;
        let cfg: Config = serde_yaml::from_str(&content)?;
        eprint!("Makanan enak {:?}", cfg);
        Ok(cfg)
    } else {
        Err("config file not found".into())
    }
}

pub fn load(file_path: &str) -> Config {
    let mut cfg = Config::default();
    cfg.load_from_env();

    match load_cfg_from_file(file_path) {
        Ok(c) => cfg = c,
        Err(err) => warn!(
            "cannot load config file: {}. using defaults. error: {:?}",
            file_path, err
        ),
    }
    cfg
}

pub enum ArgType {
    Run,
    Others,
}

pub struct Args {
    pub arg_type: ArgType,
    pub config_filename: String,
}

impl Args {
    pub fn parse() -> Self {
        let mut config_filename = "";
        let mut arg_type: ArgType = ArgType::Others;
        let matches = App::new("htmxxx")
            .version("0.1")
            .author("Fahmi Lukistriya")
            .about("HTMXxx, trying hyper rust")
            .subcommand(
                App::new("run")
                    .about("Running server")
                    .help("Running server")
                    .arg(
                        Arg::with_name("config_filename")
                            .required(false)
                            .short("c")
                            .long("config")
                            .takes_value(true)
                            .default_value("config.yml")
                            .help("Name of configuration file in *.yml format"),
                    ),
            )
            .get_matches();
        match matches.subcommand() {
            ("run", Some(init_matches)) => {
                config_filename = init_matches.value_of("config_filename").unwrap();
                arg_type = ArgType::Run;
            }
            _ => {
                println!("Invalid command. Use 'run'")
            }
        }
        Self {
            config_filename: config_filename.to_string(),
            arg_type,
        }
    }
}
