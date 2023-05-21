use std::collections::BTreeMap;

#[derive(Debug, serde::Deserialize)]
pub enum BlocklistKind {
    EtcHost,
}

impl Default for BlocklistKind {
    fn default() -> Self {
        Self::EtcHost
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct BlocklistItem {
    pub url: String,
    pub kind: BlocklistKind,
}

#[derive(Debug, Default, serde::Deserialize)]
pub struct Config {
    pub members: BTreeMap<String, BlocklistItem>,
}
