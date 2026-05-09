use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, exit};
use std::time::Duration;
use wait_timeout::ChildExt;

fn main() {
    let args: Vec<String> = env::args().collect();

    // 1. 设置 Linux 工具链路径
    let tools_path = "C:\\agent_tools"; 
    
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

/// 检查并自动下载、配置 uutils
fn setup_uutils(tools_path: &str) {
    let coreutils_exe = format!("{}\\{}", tools_path, "coreutils.exe");
    
    // 如果工具已经存在，直接返回
    if Path::new(&coreutils_exe).exists() {
        return;
    }

    eprintln!("[AgentEnv] Linux tools not found. Downloading uutils to {}...", tools_path);
    fs::create_dir_all(tools_path).expect("Failed to create agent_tools directory");

    let zip_path = format!("{}\\{}", tools_path, "coreutils.zip");
    let download_url = "https://github.com/uutils/coreutils/releases/latest/download/coreutils-x86_64-pc-windows-msvc.zip";

    // 1. 调用 Windows 内置 curl 下载
    let curl_status = Command::new("curl.exe")
        .args(["-L", download_url, "-o", &zip_path])
        .status()
        .expect("Failed to execute curl.exe");
        
    if !curl_status.success() {
        eprintln!("[AgentEnv Error] Failed to download uutils.");
        exit(1);
    }

    // 2. 调用 Windows 内置 tar 解压
    let tar_status = Command::new("tar.exe")
        .args(["-xf", &zip_path, "-C", tools_path])
        .status()
        .expect("Failed to execute tar.exe");

    if !tar_status.success() {
        eprintln!("[AgentEnv Error] Failed to extract uutils zip.");
        exit(1);
    }

    // 3. 寻找并移动 coreutils.exe
    for entry in fs::read_dir(tools_path).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            let dir_name = entry.file_name();
            if dir_name.to_string_lossy().starts_with("coreutils-") {
                let exe_in_dir = entry.path().join("coreutils.exe");
                if exe_in_dir.exists() {
                    fs::rename(exe_in_dir, &coreutils_exe).expect("Failed to move coreutils.exe");
                }
                let _ = fs::remove_dir_all(entry.path());
                break;
            }
        }
    }

    let _ = fs::remove_file(&zip_path);

    // 4. 动态解析并生成所有支持命令的批处理代理
    eprintln!("[AgentEnv] Generating command proxies...");
    generate_proxies(&coreutils_exe, tools_path);

    eprintln!("[AgentEnv] uutils setup complete.");
}

/// 运行 coreutils.exe 解析输出并动态生成所有命令的 .bat 代理
fn generate_proxies(coreutils_exe: &str, tools_path: &str) {
    // 运行 coreutils.exe 获取支持的命令列表
    let output = Command::new(coreutils_exe)
        .output()
        .expect("Failed to run coreutils.exe for proxy generation");
// uutils 通常在没有参数时将帮助信息输出到 stderr，但为了保险，我们将 stdout 和 stderr 合并
    let help_text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let mut parsing_commands = false;

    // 解析帮助文本
    for line in help_text.lines() {
        // 遇到 "Currently defined functions:" 标志位时，开始解析后续行
        if line.contains("Currently defined functions:") {
            parsing_commands = true;
            continue;
        }

        if parsing_commands {
            // 替换掉方括号和逗号，统一转换为空格以便分割
            let cleaned_line = line.replace('[', " ")
                                   .replace(']', " ")
                                   .replace(',', " ");
            
            for cmd in cleaned_line.split_whitespace() {
                let cmd_name = cmd.trim();
                
                // 过滤掉空字符串和无意义的名称
                if !cmd_name.is_empty() && cmd_name != "coreutils" {
                    let bat_path = format!("{}\\{}.bat", tools_path, cmd_name);
                    let bat_content = format!("@echo off\r\ncoreutils.exe {} %*\r\n", cmd_name);
                    
                    // 忽略写入错误，继续生成下一个
                    let _ = fs::write(&bat_path, bat_content);
                }
            }
        }
    }
}
