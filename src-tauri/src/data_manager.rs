use anyhow::{Context, Result};
use chrono::Local;
use csv::Reader;
use std::fs::{self, File};
use std::path::PathBuf;

use crate::models::SsqRecord;

const MAX_RECORDS: usize = 500;
const CSV_FILENAME: &str = "ssq_history.csv";

pub struct DataManager {
    data_dir: PathBuf,
}

impl DataManager {
    pub fn new() -> Result<Self> {
        let data_dir = Self::get_data_dir()?;
        fs::create_dir_all(&data_dir)?;
        Ok(Self { data_dir })
    }

    fn get_data_dir() -> Result<PathBuf> {
        // 获取当前可执行文件的目录，然后找到项目根目录
        let current_exe = std::env::current_exe()?;
        let exe_dir = current_exe.parent().context("无法获取可执行文件目录")?;
        
        // 在开发模式下，从 target/debug 向上找到项目根目录
        // 在发布模式下，使用可执行文件所在目录
        let project_root = if exe_dir.ends_with("target/debug") || exe_dir.ends_with("target\\debug") {
            exe_dir.parent().and_then(|p| p.parent()).unwrap_or(exe_dir)
        } else {
            exe_dir
        };
        
        Ok(project_root.to_path_buf())
    }

    pub fn get_csv_path(&self) -> PathBuf {
        self.data_dir.join(CSV_FILENAME)
    }

    /// 读取 CSV 文件的最后更新时间（从第一行注释中读取）
    pub fn get_last_update_time(&self) -> Result<Option<chrono::NaiveDate>> {
        let csv_path = self.get_csv_path();
        
        if !csv_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&csv_path)?;
        let first_line = content.lines().next();
        
        if let Some(line) = first_line {
            // 检查第一行是否是更新时间注释: # LastUpdate: 2026-02-12
            if line.starts_with("# LastUpdate: ") {
                let date_str = line.trim_start_matches("# LastUpdate: ").trim();
                if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    return Ok(Some(date));
                }
            }
        }
        
        Ok(None)
    }

    /// 加载本地历史数据
    pub fn load_local_data(&self) -> Result<Vec<SsqRecord>> {
        let csv_path = self.get_csv_path();
        
        if !csv_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&csv_path)?;
        let mut lines = content.lines();
        
        // 跳过第一行（如果是更新时间注释）
        if let Some(first_line) = lines.next() {
            if !first_line.starts_with("# LastUpdate:") {
                // 如果第一行不是注释，需要重新读取整个文件
                let file = File::open(&csv_path)?;
                let mut reader = Reader::from_reader(file);
                let mut records = Vec::new();

                for result in reader.deserialize() {
                    let record: SsqRecord = result?;
                    records.push(record);
                }

                // 保持最近500期
                if records.len() > MAX_RECORDS {
                    records = records.split_off(records.len() - MAX_RECORDS);
                }

                return Ok(records);
            }
        }
        
        // 重新构建 CSV 内容（跳过注释行）
        let csv_content_without_comment = lines.collect::<Vec<&str>>().join("\n");
        let mut reader = Reader::from_reader(csv_content_without_comment.as_bytes());
        let mut records = Vec::new();

        for result in reader.deserialize() {
            let record: SsqRecord = result?;
            records.push(record);
        }

        // 保持最近500期
        if records.len() > MAX_RECORDS {
            records = records[records.len() - MAX_RECORDS..].to_vec();
        }

        Ok(records)
    }

    /// 保存历史数据到本地
    pub fn save_local_data(&self, records: &[SsqRecord]) -> Result<()> {
        let csv_path = self.get_csv_path();
        println!("正在保存数据到: {:?}", csv_path);
        
        // 保存最近500期
        let start_index = if records.len() > MAX_RECORDS {
            records.len() - MAX_RECORDS
        } else {
            0
        };

        println!("保存 {} 条记录（从索引 {} 开始）", records.len() - start_index, start_index);
        
        // 使用 String 构建 CSV 内容，然后一次性写入
        let mut csv_content = String::new();
        
        // 添加更新时间注释（第一行）
        let today = Local::now().format("%Y-%m-%d");
        csv_content.push_str(&format!("# LastUpdate: {}\n", today));
        
        // CSV 表头
        csv_content.push_str("issue,date,red1,red2,red3,red4,red5,red6,blue_ball\n");
        
        for (idx, record) in records[start_index..].iter().enumerate() {
            if idx % 100 == 0 {
                println!("正在处理第 {} 条记录...", idx);
            }
            csv_content.push_str(&format!(
                "{},{},{},{},{},{},{},{},{}\n",
                record.issue,
                record.date,
                record.red1,
                record.red2,
                record.red3,
                record.red4,
                record.red5,
                record.red6,
                record.blue_ball
            ));
        }

        println!("CSV内容构建完成，正在写入文件...");
        std::fs::write(&csv_path, csv_content)?;
        println!("✅ CSV 文件保存成功");
        Ok(())
    }
}
