mod models;
mod data_manager;
mod fetcher;
mod analyzer;

use analyzer::Analyzer;
use data_manager::DataManager;
use fetcher::DataFetcher;
use models::{AlgorithmType, BallFrequency, PredictionResult, SsqRecord};

#[tauri::command]
fn load_and_update_data() -> Result<Vec<SsqRecord>, String> {
    let manager = DataManager::new().map_err(|e| e.to_string())?;
    
    // 1. 首先尝试加载本地 CSV 数据
    println!("正在加载本地 CSV 数据...");
    let local_records = manager.load_local_data().map_err(|e| e.to_string())?;
    
    if !local_records.is_empty() {
        let latest = local_records.last().unwrap();
        println!("本地数据已加载，共 {} 条记录", local_records.len());
        println!("最新期号: {}, 日期: {}", latest.issue, latest.date);
    } else {
        println!("本地无数据");
    }
    
    // 2. 检查最后更新时间
    let last_update = manager.get_last_update_time().map_err(|e| e.to_string())?;
    let today = chrono::Local::now().date_naive();
    
    let should_fetch = if local_records.is_empty() {
        println!("本地无数据，需要从网络获取");
        true
    } else if let Some(last_update_date) = last_update {
        if last_update_date >= today {
            println!("数据已是最新（最后更新: {}，今天: {}），无需重新获取", last_update_date, today);
            false
        } else {
            println!("数据不是最新（最后更新: {}，今天: {}），需要从网络获取", last_update_date, today);
            true
        }
    } else {
        println!("无法获取最后更新时间，需要从网络获取");
        true
    };
    
    // 3. 根据判断结果，决定最终返回的数据
    let final_records = if should_fetch {
        println!("正在从网络获取最新数据...");
        
        match DataFetcher::fetch_history(500) {
            Ok(new_records) => {
                println!("网络获取成功，获取到 {} 条记录", new_records.len());
                println!("当前本地记录数: {}", local_records.len());
                println!("开始合并数据...");
                
                // 合并并去重
                let mut merged_records = local_records;
                let mut added_count = 0;
                for new_record in new_records {
                    if !merged_records.iter().any(|r| r.issue == new_record.issue) {
                        merged_records.push(new_record);
                        added_count += 1;
                    }
                }
                println!("新增 {} 条记录", added_count);
                
                // 按期号排序
                println!("开始排序...");
                merged_records.sort_by(|a, b| a.issue.cmp(&b.issue));
                println!("排序完成");
                
                // 保存到 CSV
                println!("正在保存 {} 条记录到 CSV...", merged_records.len());
                manager.save_local_data(&merged_records).map_err(|e| e.to_string())?;
                println!("✅ 数据已更新并保存到 CSV");
                
                if let Some(latest) = merged_records.last() {
                    println!("最新数据: 期号 {}, 日期 {}", latest.issue, latest.date);
                }
                
                merged_records
            }
            Err(e) => {
                println!("网络获取失败: {}", e);
                if local_records.is_empty() {
                    return Err(format!("无本地数据且网络获取失败: {}", e));
                }
                println!("将使用现有本地数据");
                local_records
            }
        }
    } else {
        println!("使用现有本地数据");
        local_records
    };
    
    Ok(final_records)
}

#[tauri::command]
fn analyze_frequency(
    records: Vec<SsqRecord>,
    algorithm: String,
) -> Result<(Vec<BallFrequency>, Vec<BallFrequency>), String> {
    let algo_type = match algorithm.as_str() {
        "hot" => AlgorithmType::HotStaysHot,
        "cold" => AlgorithmType::ColdBounceBack,
        _ => return Err("无效的算法类型".to_string()),
    };
    
    let red_freq = Analyzer::analyze_red_frequency(&records, algo_type);
    let blue_freq = Analyzer::analyze_blue_frequency(&records, algo_type);
    
    Ok((red_freq, blue_freq))
}

#[tauri::command]
fn generate_predictions(
    records: Vec<SsqRecord>,
    algorithm: String,
) -> Result<Vec<PredictionResult>, String> {
    let algo_type = match algorithm.as_str() {
        "hot" => AlgorithmType::HotStaysHot,
        "cold" => AlgorithmType::ColdBounceBack,
        _ => return Err("无效的算法类型".to_string()),
    };
    
    let predictions = Analyzer::generate_predictions(&records, algo_type);
    Ok(predictions)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            load_and_update_data,
            analyze_frequency,
            generate_predictions
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

