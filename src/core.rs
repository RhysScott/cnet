pub mod config {
    use directories::ProjectDirs;
    use serde::{Deserialize, Serialize};
    use std::{
        error::Error,
        fmt, fs,
        path::{Path, PathBuf},
    };

    #[derive(Debug, Clone, Copy)]
    pub struct Config {
        pub base_url: &'static str,
    }

    pub static CONFIG: Config = Config {
        base_url: "http://2.2.2.2/Auth.ashx",
    };

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StoredConfig {
        pub username: String,
        pub password: String,
        pub package: String,
    }

    #[derive(Debug)]
    pub enum ConfigError {
        Io(std::io::Error),
        Invalid(serde_json::Error),
        ConfigDirUnavailable,
    }

    impl fmt::Display for ConfigError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Io(err) => write!(f, "{err}"),
                Self::Invalid(err) => write!(f, "{err}"),
                Self::ConfigDirUnavailable => write!(f, "无法确定配置文件目录"),
            }
        }
    }

    impl Error for ConfigError {}

    impl From<std::io::Error> for ConfigError {
        fn from(value: std::io::Error) -> Self {
            Self::Io(value)
        }
    }

    impl From<serde_json::Error> for ConfigError {
        fn from(value: serde_json::Error) -> Self {
            Self::Invalid(value)
        }
    }

    pub fn config_path() -> Result<PathBuf, ConfigError> {
        let dirs = ProjectDirs::from("com", "scott", "cnet")
            .ok_or(ConfigError::ConfigDirUnavailable)?;
        Ok(dirs.config_dir().join("config.json"))
    }

    pub fn load_config_if_exists() -> Result<Option<StoredConfig>, ConfigError> {
        let path = config_path()?;
        if !path.exists() {
            return Ok(None);
        }
        load_config_from_path(&path).map(Some)
    }

    pub fn save_config(config: &StoredConfig) -> Result<PathBuf, ConfigError> {
        let path = config_path()?;
        save_config_to_path(&path, config)?;
        Ok(path)
    }

    fn load_config_from_path(path: &Path) -> Result<StoredConfig, ConfigError> {
        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    fn save_config_to_path(path: &Path, config: &StoredConfig) -> Result<(), ConfigError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(config)?;
        fs::write(path, content)?;
        Ok(())
    }
}

pub mod crypto {
    use des::cipher::{BlockEncrypt, KeyInit};
    use hex::ToHex;
    use std::convert::TryInto;

    const BLOCK_SIZE: usize = 8;

    fn make_key(username: &str) -> [u8; BLOCK_SIZE] {
        let tail = if username.len() >= 4 {
            &username[username.len() - 4..]
        } else {
            username
        };

        let mut key_str = String::with_capacity(32);
        key_str.push_str(tail);
        key_str.push_str(username);
        key_str.push_str("12345678");

        let bytes = key_str.as_bytes();
        let mut key = [0u8; BLOCK_SIZE];
        let n = bytes.len().min(BLOCK_SIZE);
        key[..n].copy_from_slice(&bytes[..n]);
        key
    }

    fn pkcs7_pad(mut data: Vec<u8>) -> Vec<u8> {
        let pad_len = BLOCK_SIZE - (data.len() % BLOCK_SIZE);
        data.resize(data.len() + pad_len, pad_len as u8);
        data
    }

    pub fn encrypt_password(username: &str, password: &str) -> String {
        let key = make_key(username);
        let des = des::Des::new_from_slice(&key).expect("DES 密钥必须为 8 字节");
        
        let mut buf = pkcs7_pad(password.as_bytes().to_vec());
        
        for chunk in buf.chunks_exact_mut(BLOCK_SIZE) {
            let block_arr = chunk.try_into().expect("块大小不匹配");
            des.encrypt_block(block_arr);
        }

        buf.encode_hex()
    }
}

pub mod models {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize)]
    pub struct OnlineRequest {
        #[serde(rename = "DoWhat")]
        pub do_what: String,
        #[serde(rename = "Package")]
        pub package: String,
    }

    #[allow(dead_code)]
    #[derive(Debug, Deserialize)]
    pub struct OnlineResponse {
        #[serde(rename = "DoWhat")]
        pub operate: String,
        #[serde(rename = "Package")]
        pub network_plan: String,
        #[serde(rename = "Result")]
        pub status: bool,
        #[serde(rename = "Message")]
        pub hint: String,
    }

    #[derive(Debug, Serialize)]
    pub struct CloseNetRequest {
        #[serde(rename = "DoWhat")]
        pub do_what: String,
        #[serde(rename = "IP")]
        pub ip: String,
    }

    #[allow(dead_code)]
    #[derive(Debug, Deserialize)]
    pub struct CloseNetResponse {
        #[serde(rename = "DoWhat")]
        pub action: String,
        #[serde(rename = "IP")]
        pub ip: String,
        #[serde(rename = "Result")]
        pub status: bool,
        #[serde(rename = "Message")]
        pub message: String,
    }

    #[derive(Debug, Serialize)]
    pub struct LoginRequest {
        #[serde(rename = "DoWhat")]
        pub do_what: String,
        pub username: String,
        pub password: String,
    }

    #[derive(Debug, Serialize)]
    pub struct OperationRequest {
        #[serde(rename = "DoWhat")]
        pub do_what: String,
    }

    #[allow(dead_code)]
    #[derive(Debug, Deserialize)]
    pub struct GetInfoResponse {
        #[serde(rename = "DoWhat")]
        pub action: String,
        #[serde(rename = "Data")]
        pub data: UserData,
        #[serde(rename = "Message")]
        pub message: String,
        #[serde(rename = "Result")]
        pub success: bool,
    }

    #[allow(dead_code)]
    #[derive(Debug, Deserialize)]
    pub struct UserData {
        #[serde(rename = "IP")]
        pub ip: String,
        #[serde(rename = "MAC")]
        pub mac: String,
        #[serde(rename = "XM")]
        pub real_name: String,
        #[serde(rename = "DP")]
        pub department: String,
        #[serde(rename = "MOC")]
        pub online_count: i32,
        #[serde(rename = "KXTC")]
        pub available_packages: Vec<PackageItem>,
        #[serde(rename = "UG")]
        pub user_group: String,
        #[serde(rename = "OIA")]
        pub online_sessions: Vec<SessionInfo>,
        #[serde(rename = "CYXX")]
        pub trusted_devices: Vec<DeviceInfo>,
    }

    #[derive(Debug, Deserialize)]
    pub struct PackageItem {
        #[serde(rename = "套餐名称")]
        pub name: String,
    }

    #[allow(dead_code)]
    #[derive(Debug, Deserialize)]
    pub struct SessionInfo {
        #[serde(rename = "IP")]
        pub ip: String,
        #[serde(rename = "MAC")]
        pub mac: String,
        #[serde(rename = "VLAN")]
        pub vlan: u16,
        #[serde(rename = "Start")]
        pub start_time: String,
        #[serde(rename = "Last")]
        pub last_active: String,
        #[serde(rename = "Package")]
        pub package: PackageItem,
        #[serde(rename = "Session")]
        pub session_id: String,
    }

    #[allow(dead_code)]
    #[derive(Debug, Deserialize)]
    pub struct DeviceInfo {
        #[serde(rename = "MAC")]
        pub mac: String,
        #[serde(rename = "Desc")]
        pub description: String,
    }
}

pub mod client {
    use super::{config, crypto, models};
    use reqwest::Client;
    use std::error::Error;

    pub fn create_client() -> Result<Client, Box<dyn Error>> {
        let client = Client::builder()
            .no_proxy()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;
        Ok(client)
    }

    pub async fn login(
        client: &Client,
        username: &str,
        password: &str,
    ) -> Result<String, Box<dyn Error>> {
        let encrypted_pwd = crypto::encrypt_password(username, password);

        let req = models::LoginRequest {
            do_what: "Login".to_string(),
            username: username.to_string(),
            password: encrypted_pwd,
        };

        let resp_text = client
            .post(config::CONFIG.base_url)
            .json(&req)
            .send()
            .await?
            .text()
            .await?;

        Ok(resp_text)
    }

    pub async fn get_user_info(
        client: &Client,
    ) -> Result<models::GetInfoResponse, Box<dyn Error>> {
        let req = models::OperationRequest {
            do_what: "GetInfo".to_string(),
        };

        let resp = client
            .post(config::CONFIG.base_url)
            .json(&req)
            .send()
            .await?
            .json::<models::GetInfoResponse>()
            .await?;

        Ok(resp)
    }

    pub async fn online(
        client: &Client,
        package: &str,
    ) -> Result<models::OnlineResponse, Box<dyn Error>> {
        let req = models::OnlineRequest {
            do_what: "OpenNet".to_string(),
            package: package.to_string(),
        };

        let resp = client
            .post(config::CONFIG.base_url)
            .json(&req)
            .send()
            .await?
            .json::<models::OnlineResponse>()
            .await?;

        Ok(resp)
    }

    pub async fn offline(
        client: &Client,
        ip: &str,
    ) -> Result<models::CloseNetResponse, Box<dyn Error>> {
        let req = models::CloseNetRequest {
            do_what: "CloseNet".to_string(),
            ip: ip.to_string(),
        };

        let resp = client
            .post(config::CONFIG.base_url)
            .json(&req)
            .send()
            .await?
            .json::<models::CloseNetResponse>()
            .await?;

        Ok(resp)
    }
}
