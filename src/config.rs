use std::path::Path;

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    #[serde(default)]
    pub database: crate::service::database::Config,
    #[serde(default)]
    pub cache: crate::repository::cache::Config,
    #[serde(default)]
    pub lookup: crate::repository::lookup::Config,
    #[serde(default)]
    pub blocklists: crate::repository::blocklist::Config,
    #[serde(default)]
    pub dns: crate::dns::config::Config,
}

impl Config {
    pub fn load(path: &Path) -> Self {
        let conf = ::config::Config::builder()
            .add_source(::config::File::from(path).required(true))
            .add_source(::config::Environment::default().separator("_"))
            .build()
            .expect("unable to locate configuration file");
        conf.try_deserialize()
            .expect("configuration format invalid")
    }
}
