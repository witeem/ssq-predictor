use anyhow::{anyhow, Result};
use scraper::{Html, Selector};

use crate::models::SsqRecord;

pub struct DataFetcher;

impl DataFetcher {
    /// 从 datachart.500.com 获取双色球历史数据
    pub fn fetch_history(max_count: usize) -> Result<Vec<SsqRecord>> {
        let url = format!(
            "https://datachart.500.com/ssq/history/newinc/history.php?limit={}",
            max_count.min(500)
        );
        
        println!("正在从 {} 获取数据...", url);
        
        // 设置请求头，模拟浏览器
        let client = reqwest::blocking::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .timeout(std::time::Duration::from_secs(60))
            .build()?;
        
        match client.get(&url).send() {
            Ok(response) => {
                let html = response.text()?;
                
                // 尝试解析 HTML
                match Self::parse_html(&html, max_count) {
                    Ok(records) if !records.is_empty() => {
                        println!("成功从网络获取 {} 条记录", records.len());
                        return Ok(records);
                    }
                    Err(e) => {
                        println!("解析网页失败: {}, 使用示例数据", e);
                    }
                    _ => {
                        println!("未解析到数据，使用示例数据");
                    }
                }
            }
            Err(e) => {
                println!("网络请求失败: {}, 使用示例数据", e);
            }
        }
        
        // 如果网络获取失败，返回示例数据
        println!("提示：使用示例数据进行演示");
        Self::generate_sample_data(max_count)
    }

    fn parse_html(html: &str, max_count: usize) -> Result<Vec<SsqRecord>> {
        let document = Html::parse_document(html);
        
        // 参考实际 HTML 结构：
        // <tbody id="tdata">
        //   <tr class="t_tr1">
        //     <td>期号</td>
        //     <td class="t_cfont2">红1</td>...<td class="t_cfont2">红6</td>
        //     <td class="t_cfont4">蓝球</td>
        //     ...其他列...
        //     <td>日期</td>
        //   </tr>
        // </tbody>
        
        let selectors = vec![
            "tbody#tdata tr",
            "tbody tr.t_tr1",
            "tbody tr",
        ];
        
        let mut records = Vec::new();
        
        for selector_str in selectors {
            println!("尝试选择器: {}", selector_str);
            if let Ok(row_selector) = Selector::parse(selector_str) {
                let td_selector = Selector::parse("td").unwrap();
                
                let rows: Vec<_> = document.select(&row_selector).collect();
                println!("找到 {} 行数据", rows.len());
                
                for (row_idx, row) in rows.iter().enumerate() {
                    let cells: Vec<String> = row
                        .select(&td_selector)
                        .map(|cell| cell.text().collect::<String>().trim().to_string())
                        .collect();
                    
                    if row_idx < 3 {
                        println!("行 {}: {} 列 - 前10列: {:?}", row_idx, cells.len(), &cells[..cells.len().min(10)]);
                    }
                    
                    // 至少需要 8 列：期号(1) + 红球(6) + 蓝球(1)
                    if cells.len() < 8 {
                        continue;
                    }

                    // 第1列：期号（索引0）
                    let issue = cells[0].trim().to_string();
                    if issue.is_empty() || !issue.chars().all(|c| c.is_numeric()) {
                        continue;
                    }

                    // 第2-7列：红球（索引 1-6）
                    let mut red_balls = Vec::new();
                    let mut parse_failed = false;
                    
                    for i in 1..=6 {
                        if let Ok(num) = cells[i].parse::<u8>() {
                            if num >= 1 && num <= 33 {
                                red_balls.push(num);
                            } else {
                                parse_failed = true;
                                break;
                            }
                        } else {
                            parse_failed = true;
                            break;
                        }
                    }
                    
                    if parse_failed || red_balls.len() != 6 {
                        if row_idx < 3 {
                            println!("行 {} 红球解析失败: {:?}", row_idx, &cells[1..7]);
                        }
                        continue;
                    }

                    // 第8列：蓝球（索引 7）
                    let blue_ball = match cells[7].parse::<u8>() {
                        Ok(num) if num >= 1 && num <= 16 => num,
                        _ => {
                            if row_idx < 3 {
                                println!("行 {} 蓝球解析失败: {}", row_idx, cells[7]);
                            }
                            continue;
                        }
                    };

                    // 日期在最后一列
                    let date = if cells.len() > 10 {
                        cells[cells.len() - 1].trim().to_string()
                    } else {
                        chrono::Local::now().format("%Y-%m-%d").to_string()
                    };

                    records.push(SsqRecord::new(issue, date, red_balls, blue_ball));

                    if records.len() >= max_count {
                        break;
                    }
                }
                
                // 如果找到了记录，就不再尝试其他选择器
                if !records.is_empty() {
                    println!("✅ 使用选择器 '{}' 成功解析 {} 条记录", selector_str, records.len());
                    break;
                }
            }
        }

        if records.is_empty() {
            return Err(anyhow!("未解析到任何有效数据"));
        }

        println!("成功解析 {} 条记录", records.len());
        Ok(records)
    }

    /// 生成示例数据用于测试
    fn generate_sample_data(count: usize) -> Result<Vec<SsqRecord>> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut records = Vec::new();
        
        let base_issue = 2024001;
        
        for i in 0..count.min(500) {
            let issue = format!("{}", base_issue + i);
            let date = chrono::Local::now()
                .checked_sub_signed(chrono::Duration::days(i as i64 * 3))
                .unwrap()
                .format("%Y-%m-%d")
                .to_string();
            
            // 生成6个不重复的红球（1-33）
            let mut red_balls: Vec<u8> = Vec::new();
            while red_balls.len() < 6 {
                let num = rng.gen_range(1..=33);
                if !red_balls.contains(&num) {
                    red_balls.push(num);
                }
            }
            red_balls.sort();
            
            // 生成1个蓝球（1-16）
            let blue_ball = rng.gen_range(1..=16);
            
            records.push(SsqRecord::new(issue, date, red_balls, blue_ball));
        }
        
        // 按期号排序
        records.sort_by(|a, b| a.issue.cmp(&b.issue));
        
        Ok(records)
    }
}

