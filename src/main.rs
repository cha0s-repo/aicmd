use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, exit};
use std::time::Duration;
use wait_timeout::ChildExt;

fn main() {
    let args: Vec<String> = env::args().collect();

    // 1. Linux 工具链路径
    let tools_path = "C:\\agent_tools\\coreutils"; 
    
    // 2. 检查并自动初始化 uutils 工具环境
    setup_uutils(tools_path);

    // 获取当前的 PATH 环境变量，并将 uutils 路径置于最前端
    let current_path = env::var("PATH").unwrap_or_default();
    let new_path = format!("{};{}", tools_path, current_path);

    // 3. 初始化底层 cmd.exe 执行器
    let mut cmd = Command::new("cmd.exe");
    
    // 注入新的 PATH 和强制英文环境
    cmd.env("PATH", new_path)
       .env("LC_ALL", "C"); 

    // 4. 区分 OpenSSH 的调用模式
    if args.len() >= 3 && args[1] == "-c" {
        // 【模式 A：非交互式单次命令执行】
        let command_string = args[2..].join(" ");
        cmd.arg("/c").arg(&command_string);

        let mut child = cmd.spawn().expect("Failed to spawn command");

        let timeout = Duration::from_secs(60);
        let status = match child.wait_timeout(timeout).unwrap() {
            Some(status) => status,
            None => {
                let _ = child.kill();
                child.wait().unwrap();
                eprintln!("\n[AgentEnv Error]: Command execution timed out after 60 seconds.");
                exit(124); 
            }
        };
        
        exit(status.code().unwrap_or(1));

    } else {
        // 【模式 B：交互式持久会话】
        let mut child = cmd.spawn().expect("Failed to start interactive shell");
        let status = child.wait().expect("Interactive shell crashed");
        exit(status.code().unwrap_or(0));
    }
}

/// 检查并自动下载、配置 uutils（独立 exe 版本）
fn setup_uutils(tools_path: &str) {
    let ls_exe = format!("{}\\{}", tools_path, "ls.exe");
    
    // 如果工具已经存在，直接返回
    if Path::new(&ls_exe).exists() {
        return;
    }

    eprintln!("[AgentEnv] Linux tools not found. Downloading uutils...");
    fs::create_dir_all(tools_path).expect("Failed to create tools directory");

    let zip_path = format!("{}\\coreutils.zip", tools_path);
    let download_url = "https://github.com/uutils/coreutils/releases/latest/download/coreutils-0.8.0-x86_64-pc-windows-msvc.zip";

    // 1. 下载
    let curl_status = Command::new("curl.exe")
        .args(["-L", download_url, "-o", &zip_path])
        .status()
        .expect("Failed to execute curl.exe");
        
    if !curl_status.success() {
        eprintln!("[AgentEnv Error] Failed to download uutils.");
        exit(1);
    }

    // 2. 解压到临时目录
    let tmp_dir = format!("{}\\_tmp", tools_path);
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir).expect("Failed to create tmp dir");

    let tar_status = Command::new("tar.exe")
        .args(["-xf", &zip_path, "-C", &tmp_dir])
        .status()
        .expect("Failed to execute tar.exe");

    if !tar_status.success() {
        eprintln!("[AgentEnv Error] Failed to extract uutils zip.");
        exit(1);
    }

    // 3. 将子目录中的所有 .exe 移到 tools_path
    for entry in fs::read_dir(&tmp_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            for file in fs::read_dir(&path).unwrap() {
                let file = file.unwrap();
                let name = file.file_name();
                let name_str = name.to_string_lossy();
                if name_str.ends_with(".exe") {
                    let dest = format!("{}\\{}", tools_path, name_str);
                    let _ = fs::rename(file.path(), &dest);
                }
            }
        }
    }

    // 清理
    let _ = fs::remove_dir_all(&tmp_dir);
    let _ = fs::remove_file(&zip_path);

    eprintln!("[AgentEnv] uutils setup complete.");
}
