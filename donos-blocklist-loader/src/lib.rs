use sha2::{Digest, Sha256};
use std::collections::HashSet;

fn hash(input: &str) -> String {
    let result = Sha256::new().chain_update(input).finalize();
    base16ct::lower::encode_string(&result)
}

fn parse_hostfile(input: &str) -> HashSet<String> {
    input
        .split('\n')
        .flat_map(|line| {
            line.trim()
                .split_whitespace()
                .take_while(|item| !item.starts_with("#"))
                .enumerate()
                .filter_map(|(idx, item)| if idx > 0 { Some(item) } else { None })
                .map(|item| item.to_string())
        })
        .collect()
}

#[derive(Debug)]
pub struct Blocklist {
    pub hash: String,
    pub entries: HashSet<String>,
}

impl Blocklist {
    pub fn from_hostfile(value: &str) -> Self {
        let hash = hash(value);
        let entries = parse_hostfile(value);

        Self { hash, entries }
    }
}

pub struct BlocklistLoader;

impl BlocklistLoader {
    async fn load(&self, url: &str) -> Result<String, reqwest::Error> {
        let req = reqwest::get(url).await?;
        req.text().await
    }

    pub async fn load_hostfile(&self, url: &str) -> Result<Blocklist, reqwest::Error> {
        self.load(url)
            .await
            .map(|value| Blocklist::from_hostfile(&value))
    }
}

#[cfg(test)]
mod tests {
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
        let result = Blocklist::from_hostfile(
            r#"# nope
0.0.0.0 this.is.blocked
0.0.0.0 this.is.also.blocked blocked.again
0.0.0.0 this.is.also.blocked #Youwon'tgetthis
0.0.0.0 this.is.also.blocked # or this"#,
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
