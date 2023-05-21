use sha2::{Digest, Sha256};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum BlocklistKind {
    EtcHosts,
}

impl BlocklistKind {
    fn parse(self, input: &str) -> HashSet<String> {
        match self {
            Self::EtcHosts => parse_hostfile(input),
        }
    }
}

fn parse_hostfile(input: &str) -> HashSet<String> {
    input
        .split('\n')
        .flat_map(|line| {
            line.split_whitespace()
                .take_while(|item| !item.starts_with('#'))
                .enumerate()
                .filter_map(|(idx, item)| if idx > 0 { Some(item) } else { None })
                .map(|item| item.to_string())
        })
        .collect()
}

fn hash(input: &str) -> String {
    let result = Sha256::new().chain_update(input).finalize();
    base16ct::lower::encode_string(&result)
}

#[derive(Debug)]
pub struct Blocklist {
    pub hash: String,
    pub entries: HashSet<String>,
}

impl Blocklist {
    pub fn from_file(value: &str, kind: BlocklistKind) -> Self {
        let hash = hash(value);
        let entries = kind.parse(value);

        Self { hash, entries }
    }
}

#[derive(Debug, Default)]
pub struct BlocklistLoader;

impl BlocklistLoader {
    pub async fn load(&self, url: &str, kind: BlocklistKind) -> Result<Blocklist, reqwest::Error> {
        tracing::debug!("loading {url:?}");
        let req = reqwest::get(url).await?;
        let text = req.text().await?;
        Ok(Blocklist::from_file(&text, kind))
    }
}

#[cfg(test)]
mod tests {
    use crate::BlocklistKind;

    use super::{hash, parse_hostfile, Blocklist};

    #[test]
    fn parse_ads_hostfile() {
        let data = include_str!("../data/ads.txt");
        let result = parse_hostfile(data);
        assert!(result.contains("0.r.msn.com"));
        assert!(result.contains("207.net"));
        assert!(!result.contains("#"));
        assert!(!result.contains("0.0.0.0"));
    }

    #[test]
    fn parse_basic_hostfile() {
        let data = include_str!("../data/basic.txt");
        let result = parse_hostfile(data);
        assert!(result.contains("0-app.com"));
        assert!(!result.contains("#"));
        assert!(!result.contains("0.0.0.0"));
        assert_eq!(
            hash(data),
            "c0d1929bb2584c045eece5cf9d46ae913fc524e960893ab469f8a93a88fe6e94"
        );
    }

    #[test]
    fn parse_complex() {
        let result = Blocklist::from_file(
            r#"# nope
0.0.0.0 this.is.blocked
0.0.0.0 this.is.also.blocked blocked.again
0.0.0.0 this.is.also.blocked #Youwon'tgetthis
0.0.0.0 this.is.also.blocked # or this"#,
            BlocklistKind::EtcHosts,
        );
        assert!(result.entries.contains("this.is.blocked"));
        assert!(result.entries.contains("this.is.also.blocked"));
        assert!(result.entries.contains("blocked.again"));
        assert!(!result.entries.contains("nope"));
        assert!(!result.entries.contains("Youwon'tgetthis"));
        assert!(!result.entries.contains("or"));
        assert!(!result.entries.contains("this"));
        assert_eq!(
            result.hash,
            "52139cfb54f4ca549444fe7cf31b30a6f71174dc39eeaf2df631ebd34b91950d"
        );
    }
}
