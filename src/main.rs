mod core;

use clap::{Parser, Subcommand};
use dialoguer::{Confirm, Input, Select};
use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::core::{
    client,
    config::{self, ConfigError, StoredConfig},
};

#[derive(Parser)]
#[command(name = "cnet")]
#[command(about = "校园网认证工具")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Setup,
    Offline,
    AddToPath,
}

#[cfg(windows)]
fn pause_console() {
    use std::io;
    println!("\n按回车键关闭窗口...");
    let mut buf = String::new();
    let _ = io::stdin().read_line(&mut buf);
}

#[cfg(not(windows))]
fn pause_console() {}

#[tokio::main]
async fn main() {
    let exit_code = match run().await {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("错误: {err}");
            1
        }
    };

    pause_console();
    std::process::exit(exit_code);
}

async fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Setup) => {
            let config = setup_config().await?;
            println!("配置已保存: {}", config::config_path()?.display());
            println!("当前套餐: {}", config.package);
        }
        Some(Commands::Offline) => {
            let config = ensure_config().await?;
            disconnect_network(&config).await?;
        }
        Some(Commands::AddToPath) => {
            add_current_executable_to_path()?;
        }
        None => {
            let config = ensure_config().await?;
            connect_network(&config).await?;
        }
    }

    Ok(())
}

async fn ensure_config() -> Result<StoredConfig, Box<dyn Error>> {
    match config::load_config_if_exists() {
        Ok(Some(config)) => Ok(config),
        Ok(None) => {
            println!("未找到配置，开始初始化。");
            setup_config().await
        }
        Err(ConfigError::Invalid(_)) => {
            let path = config::config_path()?;
            println!("配置文件已损坏: {}", path.display());
            if Confirm::new().with_prompt("是否覆盖并重新生成配置？").interact()? {
                setup_config().await
            } else {
                Err("配置文件损坏，已取消操作。".into())
            }
        }
        Err(err) => Err(err.into()),
    }
}

async fn setup_config() -> Result<StoredConfig, Box<dyn Error>> {
    let existing = match config::load_config_if_exists() {
        Ok(value) => value,
        Err(ConfigError::Invalid(_)) => {
            let path = config::config_path()?;
            println!("配置文件已损坏: {}", path.display());
            if Confirm::new().with_prompt("是否覆盖并重新生成配置？").interact()? {
                None
            } else {
                return Err("配置文件损坏，已取消操作。".into());
            }
        }
        Err(err) => return Err(err.into()),
    };

    let username = prompt_username(existing.as_ref())?;
    let password = prompt_password(existing.as_ref())?;
    let package = prompt_package(&username, &password, existing.as_ref()).await?;

    let stored = StoredConfig {
        username,
        password,
        package,
    };

    let path = config::save_config(&stored)?;
    println!("配置已保存到 {}", path.display());

    if Confirm::new()
        .with_prompt("是否把 cnet 添加到当前用户 PATH？")
        .interact()?
    {
        if let Err(err) = add_current_executable_to_path() {
            eprintln!("添加到 PATH 失败: {err}");
        }
    }

    Ok(stored)
}

fn prompt_username(existing: Option<&StoredConfig>) -> Result<String, Box<dyn Error>> {
    let mut prompt = Input::<String>::new().with_prompt("账号");
    if let Some(current) = existing {
        prompt = prompt.allow_empty(true).with_initial_text(&current.username);
    }

    let value = prompt.interact_text()?;
    if value.trim().is_empty() {
        if let Some(current) = existing {
            return Ok(current.username.clone());
        }
    }

    Ok(value.trim().to_string())
}

fn prompt_password(existing: Option<&StoredConfig>) -> Result<String, Box<dyn Error>> {
    let mut prompt = Input::<String>::new().with_prompt("密码");
    if existing.is_some() {
        prompt = prompt.allow_empty(true);
    }

    let value = prompt.interact_text()?;
    if value.is_empty() {
        if let Some(current) = existing {
            return Ok(current.password.clone());
        }
    }

    Ok(value)
}

async fn prompt_package(
    username: &str,
    password: &str,
    existing: Option<&StoredConfig>,
) -> Result<String, Box<dyn Error>> {
    let packages = fetch_packages(username, password).await;

    if let Ok(list) = packages {
        if !list.is_empty() {
            let mut items = list.clone();
            items.push("手动输入套餐".to_string());

            let default_index = existing
                .and_then(|config| list.iter().position(|item| item == &config.package))
                .unwrap_or(0);

            let selection = Select::new()
                .with_prompt("选择套餐")
                .items(&items)
                .default(default_index)
                .interact()?;

            if selection < list.len() {
                return Ok(list[selection].clone());
            }
        }
    } else {
        println!("自动获取套餐失败，将改为手动输入。");
    }

    prompt_package_manually(existing)
}

fn prompt_package_manually(existing: Option<&StoredConfig>) -> Result<String, Box<dyn Error>> {
    let mut prompt = Input::<String>::new().with_prompt("套餐");
    if let Some(current) = existing {
        prompt = prompt.allow_empty(true).with_initial_text(&current.package);
    }

    let value = prompt.interact_text()?;
    if value.trim().is_empty() {
        if let Some(current) = existing {
            return Ok(current.package.clone());
        }
    }

    Ok(value.trim().to_string())
}

async fn fetch_packages(username: &str, password: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let client = client::create_client()?;
    client::login(&client, username, password).await?;
    let info = client::get_user_info(&client).await?;

    Ok(info
        .data
        .available_packages
        .into_iter()
        .map(|item| item.name)
        .collect())
}

async fn connect_network(config: &StoredConfig) -> Result<(), Box<dyn Error>> {
    let client = client::create_client()?;
    client::login(&client, &config.username, &config.password).await?;
    let info = client::get_user_info(&client).await?;
    let response = client::online(&client, &config.package).await?;

    if response.status {
        println!("认证成功: {}", response.hint);
        println!("IP: {}", info.data.ip);
        println!("套餐: {}", config.package);
    } else {
        println!("认证失败: {}", response.hint);
    }

    Ok(())
}

async fn disconnect_network(config: &StoredConfig) -> Result<(), Box<dyn Error>> {
    let client = client::create_client()?;
    client::login(&client, &config.username, &config.password).await?;
    let info = client::get_user_info(&client).await?;
    let response = client::offline(&client, &info.data.ip).await?;

    if response.status {
        println!("下线成功: {}", response.message);
        println!("IP: {}", info.data.ip);
    } else {
        println!("下线失败: {}", response.message);
    }

    Ok(())
}

fn add_current_executable_to_path() -> Result<(), Box<dyn Error>> {
    let exe_dir = current_executable_dir()?;
    if current_path_contains(&exe_dir) {
        println!("当前目录已在 PATH 中: {}", exe_dir.display());
        return Ok(());
    }

    if cfg!(target_os = "windows") {
        add_to_path_windows(&exe_dir)?;
        println!("已添加到当前用户 PATH，请重新打开终端后再使用 cnet。");
    } else {
        let profile = add_to_path_unix(&exe_dir)?;
        println!(
            "已写入 {}，请重新打开终端或执行 `source {}` 后再使用 cnet。",
            profile.display(),
            profile.display()
        );
    }

    Ok(())
}

fn current_executable_dir() -> Result<PathBuf, Box<dyn Error>> {
    let exe = env::current_exe()?;
    let dir = exe
        .parent()
        .ok_or("无法确定当前可执行文件所在目录")?;
    Ok(dir.to_path_buf())
}

fn current_path_contains(dir: &Path) -> bool {
    env::var_os("PATH")
        .map(|value| env::split_paths(&value).any(|item| item == dir))
        .unwrap_or(false)
}

fn add_to_path_windows(dir: &Path) -> Result<(), Box<dyn Error>> {
    let current = env::var("PATH").unwrap_or_default();
    let dir_str = dir.to_string_lossy();
    let new_path = if current.is_empty() {
        dir_str.into_owned()
    } else {
        format!("{current};{dir_str}")
    };

    let status = Command::new("setx").arg("PATH").arg(&new_path).status()?;
    if !status.success() {
        return Err("setx 执行失败".into());
    }

    Ok(())
}

fn add_to_path_unix(dir: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let home = env::var_os("HOME").ok_or("无法确定用户主目录")?;
    let profile = PathBuf::from(home).join(".zprofile");
    let line = format!("export PATH=\"{}:$PATH\"", dir.display());

    let existing = fs::read_to_string(&profile).unwrap_or_default();
    if !existing.lines().any(|entry| entry == line) {
        let content = if existing.is_empty() {
            format!("{line}\n")
        } else if existing.ends_with('\n') {
            format!("{existing}{line}\n")
        } else {
            format!("{existing}\n{line}\n")
        };
        fs::write(&profile, content)?;
    }

    Ok(profile)
}
