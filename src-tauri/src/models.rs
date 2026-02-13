use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::ser::SerializeStruct;
use serde::de::{self, MapAccess, Visitor};
use chrono::NaiveDate;
use std::fmt;

/// 双色球历史记录
#[derive(Debug, Clone)]
pub struct SsqRecord {
    /// 期号
    pub issue: String,
    /// 开奖日期
    pub date: String,
    /// 红球1
    pub red1: u8,
    /// 红球2
    pub red2: u8,
    /// 红球3
    pub red3: u8,
    /// 红球4
    pub red4: u8,
    /// 红球5
    pub red5: u8,
    /// 红球6
    pub red6: u8,
    /// 蓝球
    pub blue_ball: u8,
}

// 自定义序列化，为 JSON 添加 red_balls 数组
impl Serialize for SsqRecord {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("SsqRecord", 4)?;
        state.serialize_field("issue", &self.issue)?;
        state.serialize_field("date", &self.date)?;
        state.serialize_field("red_balls", &self.red_balls())?;
        state.serialize_field("blue_ball", &self.blue_ball)?;
        state.end()
    }
}

// 自定义反序列化，支持两种格式
impl<'de> Deserialize<'de> for SsqRecord {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Issue,
            Date,
            RedBalls,
            Red1,
            Red2,
            Red3,
            Red4,
            Red5,
            Red6,
            BlueBall,
        }

        struct SsqRecordVisitor;

        impl<'de> Visitor<'de> for SsqRecordVisitor {
            type Value = SsqRecord;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct SsqRecord")
            }

            fn visit_map<V>(self, mut map: V) -> Result<SsqRecord, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut issue = None;
                let mut date = None;
                let mut red_balls: Option<Vec<u8>> = None;
                let mut red1 = None;
                let mut red2 = None;
                let mut red3 = None;
                let mut red4 = None;
                let mut red5 = None;
                let mut red6 = None;
                let mut blue_ball = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Issue => {
                            issue = Some(map.next_value()?);
                        }
                        Field::Date => {
                            date = Some(map.next_value()?);
                        }
                        Field::RedBalls => {
                            red_balls = Some(map.next_value()?);
                        }
                        Field::Red1 => {
                            red1 = Some(map.next_value()?);
                        }
                        Field::Red2 => {
                            red2 = Some(map.next_value()?);
                        }
                        Field::Red3 => {
                            red3 = Some(map.next_value()?);
                        }
                        Field::Red4 => {
                            red4 = Some(map.next_value()?);
                        }
                        Field::Red5 => {
                            red5 = Some(map.next_value()?);
                        }
                        Field::Red6 => {
                            red6 = Some(map.next_value()?);
                        }
                        Field::BlueBall => {
                            blue_ball = Some(map.next_value()?);
                        }
                    }
                }

                let issue = issue.ok_or_else(|| de::Error::missing_field("issue"))?;
                let date = date.ok_or_else(|| de::Error::missing_field("date"))?;
                let blue_ball = blue_ball.ok_or_else(|| de::Error::missing_field("blue_ball"))?;

                // 支持两种格式：red_balls 数组或 red1-red6 独立字段
                let (r1, r2, r3, r4, r5, r6) = if let Some(balls) = red_balls {
                    if balls.len() != 6 {
                        return Err(de::Error::custom("red_balls must contain exactly 6 elements"));
                    }
                    (balls[0], balls[1], balls[2], balls[3], balls[4], balls[5])
                } else {
                    (
                        red1.ok_or_else(|| de::Error::missing_field("red1"))?,
                        red2.ok_or_else(|| de::Error::missing_field("red2"))?,
                        red3.ok_or_else(|| de::Error::missing_field("red3"))?,
                        red4.ok_or_else(|| de::Error::missing_field("red4"))?,
                        red5.ok_or_else(|| de::Error::missing_field("red5"))?,
                        red6.ok_or_else(|| de::Error::missing_field("red6"))?,
                    )
                };

                Ok(SsqRecord {
                    issue,
                    date,
                    red1: r1,
                    red2: r2,
                    red3: r3,
                    red4: r4,
                    red5: r5,
                    red6: r6,
                    blue_ball,
                })
            }
        }

        const FIELDS: &[&str] = &["issue", "date", "red_balls", "red1", "red2", "red3", "red4", "red5", "red6", "blue_ball"];
        deserializer.deserialize_struct("SsqRecord", FIELDS, SsqRecordVisitor)
    }
}

impl SsqRecord {
    pub fn new(issue: String, date: String, red_balls: Vec<u8>, blue_ball: u8) -> Self {
        assert_eq!(red_balls.len(), 6, "红球必须是6个");
        Self {
            issue,
            date,
            red1: red_balls[0],
            red2: red_balls[1],
            red3: red_balls[2],
            red4: red_balls[3],
            red5: red_balls[4],
            red6: red_balls[5],
            blue_ball,
        }
    }

    pub fn get_date(&self) -> Option<NaiveDate> {
        NaiveDate::parse_from_str(&self.date, "%Y-%m-%d").ok()
    }

    pub fn red_balls(&self) -> Vec<u8> {
        vec![self.red1, self.red2, self.red3, self.red4, self.red5, self.red6]
    }
}

/// 球号频率统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BallFrequency {
    pub number: u8,
    pub frequency: usize,
    pub weight: f64,
}

/// 预测算法类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AlgorithmType {
    /// 热号恒热
    HotStaysHot,
    /// 冷号反弹
    ColdBounceBack,
}

/// 预测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    /// 红球
    pub red_balls: Vec<u8>,
    /// 蓝球
    pub blue_ball: u8,
    /// 得分（置信度）
    pub score: f64,
}
