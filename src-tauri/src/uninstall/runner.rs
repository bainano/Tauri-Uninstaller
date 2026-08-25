use serde::Serialize;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// 卸载命令解析结果
#[derive(Clone, Debug)]
pub struct ParsedCommand {
    /// 可执行文件路径（含引号处理后的程序）
    pub program: String,
    /// 参数列表
    pub args: Vec<String>,
    /// 是否为 MSI 卸载（需要转为 /X GUID 静默）
    pub is_msi: bool,
    /// MSI ProductCode
    pub msi_code: Option<String>,
    /// 原始字符串
    pub raw: String,
}

/// 卸载执行结果
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UninstallOutcome {
    /// started / failed / timed_out / finished
    pub status: String,
    /// 进程 PID（未启动时为 None）
    pub pid: Option<u32>,
    /// 退出码（正常结束时）
    pub exit_code: Option<i32>,
    /// 人类可读消息
    pub message: String,
    /// 等待时长（秒）
    pub waited_secs: u64,
}

/// 解析 UninstallString，拆分程序与参数，识别 MSI
pub fn parse_uninstall_string(raw: &str) -> ParsedCommand {
    let trimmed = raw.trim();
    let mut parsed = ParsedCommand {
        program: String::new(),
        args: Vec::new(),
        is_msi: false,
        msi_code: None,
        raw: trimmed.to_string(),
    };
    if trimmed.is_empty() {
        return parsed;
    }

    // 提取 MSI 特征：msiexec /I{GUID} /X{GUID} /i {GUID} /x {GUID}
    let lower = trimmed.to_lowercase();
    let msi_guid = extract_msi_guid(trimmed);
    if (lower.contains("msiexec") || trimmed.to_lowercase().contains("msiexec.exe")) && msi_guid.is_some() {
        parsed.is_msi = true;
        parsed.msi_code = msi_guid.clone();
        parsed.program = "msiexec".to_string();
        parsed.args = vec![
            "/X".to_string(),
            msi_guid.unwrap(),
            "/qn".to_string(),
            "/norestart".to_string(),
        ];
        return parsed;
    }

    // 通用拆分：优先处理引号包裹的程序路径
    let chars: Vec<char> = trimmed.chars().collect();
    let mut i = 0;
    let mut token = String::new();
    let mut tokens: Vec<String> = Vec::new();
    let mut in_quote = false;
    while i < chars.len() {
        let c = chars[i];
        match c {
            '"' => {
                if in_quote {
                    in_quote = false;
                } else {
                    in_quote = true;
                }
                token.push('"');
            }
            ' ' | '\t' => {
                if in_quote {
                    token.push(c);
                } else if !token.is_empty() {
                    tokens.push(token.clone());
                    token.clear();
                }
            }
            _ => token.push(c),
        }
        i += 1;
    }
    if !token.is_empty() {
        tokens.push(token);
    }
    if tokens.is_empty() {
        return parsed;
    }

    parsed.program = tokens[0].trim_matches('"').to_string();
    parsed.args = tokens[1..].to_vec();
    parsed
}

/// 从字符串中提取 MSI ProductCode（{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}）
fn extract_msi_guid(s: &str) -> Option<String> {
    let start = s.find('{')?;
    let end = s.find('}')?;
    if end <= start {
        return None;
    }
    let guid = &s[start..=end];
    // 粗略校验格式
    let inner = guid.trim_matches('{').trim_matches('}');
    let parts: Vec<&str> = inner.split('-').collect();
    if parts.len() == 5 && parts.iter().all(|p| !p.is_empty()) {
        Some(guid.to_string())
    } else {
        None
    }
}

/// 为给定程序/参数追加静默参数（针对常见安装框架）
pub fn append_silent_args(parsed: &ParsedCommand) -> Vec<String> {
    let mut args = parsed.args.clone();
    if parsed.is_msi {
        return args; // 已含 /qn
    }
    let program_lower = parsed
        .program
        .rsplit('\\')
        .next()
        .unwrap_or("")
        .to_lowercase();
    let joined = args.join(" ").to_lowercase();

    // 已有明显静默参数则不再追加
    let silent_hint = ["/s", "/silent", "/quiet", "-silent", "-quiet", "/verysilent", "/qn", "/qb", "-s"];
    if silent_hint.iter().any(|h| {
        joined.split_whitespace().any(|t| t.to_lowercase() == *h) || joined.contains(h)
    }) {
        return args;
    }

    // Inno Setup
    if joined.contains("innosetup") || joined.contains("/verysilent") {
        args.push("/VERYSILENT".into());
        args.push("/SUPPRESSMSGBOXES".into());
        args.push("/NORESTART".into());
    }
    // NSIS
    else if joined.contains("nsis") || program_lower.contains("uninst") {
        args.push("/S".into());
    }
    // WiX Burn
    else if joined.contains("burn") || joined.contains("-burn") {
        args.push("-quiet".into());
    }
    // 通用兜底：追加 /S（多数卸载器接受）
    else {
        args.push("/S".into());
    }
    args
}

/// 启动卸载进程（默认追加静默参数）
pub fn spawn_uninstaller(parsed: &ParsedCommand, silent: bool) -> Result<Child, String> {
    let mut args = parsed.args.clone();
    if silent && !parsed.is_msi {
        args = append_silent_args(parsed);
    }
    let mut cmd = Command::new(&parsed.program);
    cmd.args(&args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::null());
    // MSI 需要显式指定 System32 下的 msiexec
    if parsed.is_msi {
        let sys = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into());
        cmd = Command::new(format!("{}\\System32\\msiexec.exe", sys));
        cmd.args(&args);
    }
    cmd.spawn().map_err(|e| format!("启动卸载器失败: {}", e))
}

/// 等待进程结束，带超时。返回 (是否超时, 退出码)
/// silent_timeout 用于静默卸载的等待窗口（默认 30s 内若无 GUI 副进程则视为已退出）
pub fn wait_with_timeout(
    child: &mut Child,
    timeout: Duration,
) -> Result<(bool, Option<i32>), String> {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return Ok((false, status.code()));
            }
            Ok(None) => {
                if start.elapsed() >= timeout {
                    return Ok((true, None));
                }
                std::thread::sleep(Duration::from_millis(500));
            }
            Err(e) => return Err(format!("等待卸载进程失败: {}", e)),
        }
    }
}

/// 强制终止进程（含子进程树，Windows taskkill /T /F）
pub fn kill_process_tree(pid: u32) -> Result<(), String> {
    let out = Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .output()
        .map_err(|e| format!("调用 taskkill 失败: {}", e))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}
