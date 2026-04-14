use std::fs;
use std::path::PathBuf;

/// 清理超过限制的旧日志文件
pub fn cleanup_old_logs(
    directory: &str,
    filename: &str,
    count_limit: u32,
) -> Result<(), std::io::Error> {
    if count_limit == 0 {
        return Ok(()); // 0 表示不限制
    }

    let dir_path = PathBuf::from(directory);
    if !dir_path.exists() {
        return Ok(());
    }

    // 获取所有符合模式的日志文件
    let mut log_files: Vec<(u32, PathBuf)> = Vec::new();
    
    if let Ok(entries) = fs::read_dir(&dir_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Some(name) = entry.file_name().to_str() {
                    // 主日志文件
                    if name == format!("{}.log", filename) {
                        log_files.push((0, entry.path()));
                    }
                    // 备份文件格式: app.log.1, app.log.2, app.log.2026-02-06, 等
                    if name.starts_with(&format!("{}.", filename)) || name.starts_with(&format!("{}-", filename)) {
                        if let Some(index_str) = name.strip_prefix(&format!("{}.", filename)) {
                            // 尝试解析数字索引
                            if let Ok(index) = index_str.parse::<u32>() {
                                log_files.push((index, entry.path()));
                            }
                        }
                    }
                }
            }
        }
    }

    // 按索引排序，保留最新的 count_limit 个
    log_files.sort_by_key(|k| k.0);
    
    if log_files.len() > count_limit as usize {
        let to_delete = log_files.len() - count_limit as usize;
        for (_, path) in log_files.iter().take(to_delete) {
            let _ = fs::remove_file(path);
        }
    }

    Ok(())
}

/// 检查日志文件大小并返回是否超过限制
pub fn check_log_file_size(
    directory: &str,
    filename: &str,
    size_limit_mb: u64,
) -> Result<bool, std::io::Error> {
    if size_limit_mb == 0 {
        return Ok(false); // 0 表示不限制
    }

    let log_path = PathBuf::from(directory).join(format!("{}.log", filename));
    
    if log_path.exists() {
        let size_bytes = fs::metadata(&log_path)?.len();
        let size_mb = size_bytes / (1024 * 1024);
        Ok(size_mb > size_limit_mb)
    } else {
        Ok(false)
    }
}

