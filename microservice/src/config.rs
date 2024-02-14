use std::{env::var, net::Ipv4Addr, str::FromStr};

#[derive(Debug, Clone)]
#[allow(non_snake_case)]
pub struct Config {
    pub RUST_LOG: String,
    pub SERVER_IP: Ipv4Addr,
    pub SERVER_PORT: u16,
    pub DATABASE_URL: String,
    pub DATABASE_MAX_CONNECTIONS: u8,
    pub ADMIN_TOKEN: String,
    pub SALT: String,
}

impl Config {
    #[allow(non_snake_case)]
    pub fn load_from_env() -> anyhow::Result<Self> {
        let RUST_LOG = var("RUST_LOG").expect("`RUST_LOG` is not set");
        let SERVER_IP = var("SERVER_IP").expect("`SERVER_IP` is not set");
        let SERVER_PORT = var("SERVER_PORT").expect("`SERVER_PORT` is not set");
        let DB_URL = var("DATABASE_URL").expect("`DB_URL` is not set");
        let DB_MAX_CONNECTIONS =
            var("DATABASE_MAX_CONNECTIONS").expect("`DB_MAX_CONNECTIONS` is not set");
        let ADMIN_TOKEN = var("ADMIN_TOKEN").expect("`ADMIN_TOKEN` is not set");
        let SALT = var("SALT").expect("`SALT` is not set");

        let SERVER_IP = Config::parse_ip(SERVER_IP).expect("IP Address wrong format");
        let SERVER_PORT = SERVER_PORT.parse::<u16>()?;
        let DB_MAX_CONNECTIONS = DB_MAX_CONNECTIONS.parse::<u8>()?;

        Ok(Self {
            RUST_LOG,
            SERVER_IP,
            SERVER_PORT,
            DATABASE_URL: DB_URL,
            DATABASE_MAX_CONNECTIONS: DB_MAX_CONNECTIONS,
            ADMIN_TOKEN,
            SALT,
        })
    }

    fn parse_ip(ip_string: String) -> anyhow::Result<Ipv4Addr> {
        Ok(Ipv4Addr::from_str(&ip_string)?)
    }
}
