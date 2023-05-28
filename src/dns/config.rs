use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    #[serde(default = "Config::default_host")]
    pub host: IpAddr,
    #[serde(default = "Config::default_port")]
    pub port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: Self::default_host(),
            port: Self::default_port(),
        }
    }
}

impl Config {
    fn default_host() -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
    }

    fn default_port() -> u16 {
        53
    }
}

impl Config {
    pub fn address(&self) -> SocketAddr {
        SocketAddr::from((self.host, self.port))
    }
}
