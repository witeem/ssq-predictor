use std::collections::HashMap;
use rand::Rng;

use crate::models::{AlgorithmType, BallFrequency, PredictionResult, SsqRecord};

const RED_BALL_MIN: u8 = 1;
const RED_BALL_MAX: u8 = 33;
const BLUE_BALL_MIN: u8 = 1;
const BLUE_BALL_MAX: u8 = 16;
const PREDICTION_COUNT: usize = 10;
const ITERATION_COUNT: usize = 10000;

pub struct Analyzer;

impl Analyzer {
    /// 分析红球频率
    pub fn analyze_red_frequency(
        records: &[SsqRecord],
        algorithm: AlgorithmType,
    ) -> Vec<BallFrequency> {
        let mut frequency_map: HashMap<u8, usize> = HashMap::new();

        // 统计每个号码出现次数
        for record in records {
            for &ball in &record.red_balls() {
                *frequency_map.entry(ball).or_insert(0) += 1;
            }
        }

        // 计算权重
        let mut frequencies: Vec<BallFrequency> = (RED_BALL_MIN..=RED_BALL_MAX)
            .map(|num| {
                let freq = *frequency_map.get(&num).unwrap_or(&0);
                let weight = Self::calculate_weight(freq, records.len(), algorithm);
                BallFrequency {
                    number: num,
                    frequency: freq,
                    weight,
                }
            })
            .collect();

        frequencies.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        frequencies
    }

    /// 分析蓝球频率
    pub fn analyze_blue_frequency(
        records: &[SsqRecord],
        algorithm: AlgorithmType,
    ) -> Vec<BallFrequency> {
        let mut frequency_map: HashMap<u8, usize> = HashMap::new();

        // 统计每个号码出现次数
        for record in records {
            *frequency_map.entry(record.blue_ball).or_insert(0) += 1;
        }

        // 计算权重
        let mut frequencies: Vec<BallFrequency> = (BLUE_BALL_MIN..=BLUE_BALL_MAX)
            .map(|num| {
                let freq = *frequency_map.get(&num).unwrap_or(&0);
                let weight = Self::calculate_weight(freq, records.len(), algorithm);
                BallFrequency {
                    number: num,
                    frequency: freq,
                    weight,
                }
            })
            .collect();

        frequencies.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        println!("Blue Frequencies: {:?}", frequencies);
        frequencies
    }

    /// 计算权重
    fn calculate_weight(frequency: usize, total_records: usize, algorithm: AlgorithmType) -> f64 {
        if total_records == 0 {
            return 0.0;
        }

        let base_probability = frequency as f64 / total_records as f64;

        match algorithm {
            // 热号恒热：频率越高，权重越大
            AlgorithmType::HotStaysHot => {
                // 使用平方函数增强热号权重
                base_probability * base_probability * 100.0
            }
            // 冷号反弹：频率越低，权重越大
            AlgorithmType::ColdBounceBack => {
                // 反转权重，频率低的权重高
                let inverted = 1.0 - base_probability;
                inverted * inverted * 100.0
            }
        }
    }

    /// 生成预测结果
    pub fn generate_predictions(
        records: &[SsqRecord],
        algorithm: AlgorithmType,
    ) -> Vec<PredictionResult> {
        let red_frequencies = Self::analyze_red_frequency(records, algorithm);
        let blue_frequencies = Self::analyze_blue_frequency(records, algorithm);

        let mut rng = rand::thread_rng();
        let mut predictions = Vec::new();

        // 进行多次迭代，选出最优的组合
        for _ in 0..ITERATION_COUNT {
            // 基于权重随机选择红球
            let red_balls = Self::weighted_random_selection(&red_frequencies, 6, &mut rng);
            
            // 基于权重随机选择蓝球
            let blue_ball = Self::weighted_random_selection(&blue_frequencies, 1, &mut rng)[0];

            // 计算得分
            let score = Self::calculate_score(&red_balls, blue_ball, &red_frequencies, &blue_frequencies);

            predictions.push(PredictionResult {
                red_balls,
                blue_ball,
                score,
            });
        }

        // 按得分排序
        predictions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // 去重并返回前10个
        let mut unique_predictions = Vec::new();
        for pred in predictions {
            if !unique_predictions.iter().any(|p: &PredictionResult| {
                Self::is_same_prediction(p, &pred)
            }) {
                unique_predictions.push(pred);
                if unique_predictions.len() >= PREDICTION_COUNT {
                    break;
                }
            }
        }

        unique_predictions
    }

    /// 基于权重的随机选择
    fn weighted_random_selection(
        frequencies: &[BallFrequency],
        count: usize,
        rng: &mut impl Rng,
    ) -> Vec<u8> {
        let total_weight: f64 = frequencies.iter().map(|f| f.weight).sum();
        let mut selected = Vec::new();
        let mut available: Vec<_> = frequencies.to_vec();

        while selected.len() < count && !available.is_empty() {
            let rand_value = rng.gen::<f64>() * total_weight;
            let mut cumulative = 0.0;

            for (i, freq) in available.iter().enumerate() {
                cumulative += freq.weight;
                if rand_value <= cumulative {
                    selected.push(freq.number);
                    available.remove(i);
                    break;
                }
            }

            // 如果权重太低导致无法选中，随机选一个
            if selected.len() < count && available.len() > 0 {
                let idx = rng.gen_range(0..available.len());
                selected.push(available[idx].number);
                available.remove(idx);
            }
        }

        selected.sort();
        selected
    }

    /// 计算得分
    fn calculate_score(
        red_balls: &[u8],
        blue_ball: u8,
        red_frequencies: &[BallFrequency],
        blue_frequencies: &[BallFrequency],
    ) -> f64 {
        let mut score = 0.0;

        // 红球得分
        for &ball in red_balls {
            if let Some(freq) = red_frequencies.iter().find(|f| f.number == ball) {
                score += freq.weight;
            }
        }

        // 蓝球得分
        if let Some(freq) = blue_frequencies.iter().find(|f| f.number == blue_ball) {
            score += freq.weight;
        }

        score
    }

    /// 判断是否为相同的预测
    fn is_same_prediction(a: &PredictionResult, b: &PredictionResult) -> bool {
        a.red_balls == b.red_balls && a.blue_ball == b.blue_ball
    }
}
