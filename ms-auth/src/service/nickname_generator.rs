// 昵称生成器
// 高性能、低重复率、自然的昵称生成

use rand::Rng;

/// 昵称生成模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NicknameMode {
    /// 纯中文
    ChineseOnly,
    /// 纯英文
    EnglishOnly,
    /// 中英文混合
    Mixed,
    /// 自动选择（随机）
    Auto,
}

/// 昵称生成器配置
#[derive(Debug, Clone)]
pub struct NicknameConfig {
    /// 生成模式
    pub mode: NicknameMode,
    /// 是否添加数字后缀（数字只能结尾）
    pub with_number: bool,
    /// 最大长度（字符数，不超过32）
    pub max_length: usize,
}

impl Default for NicknameConfig {
    fn default() -> Self {
        Self {
            mode: NicknameMode::Auto,
            with_number: false,
            max_length: 16,
        }
    }
}

/// 昵称生成器
pub struct NicknameGenerator;

// 使用 Mutex 确保线程安全的随机数生成器
// 每次生成时创建新的随机数生成器，避免全局状态

// 中文词库
static CHINESE_ADJECTIVES: &[&str] = &[
    "优雅", "勇敢", "聪明", "温柔", "活泼", "安静", "热情", "冷静", "幽默", "认真", "自由", "快乐",
    "阳光", "温暖", "清新", "浪漫", "神秘", "独特", "精致", "自然", "梦幻", "纯真", "成熟", "年轻",
    "活力", "沉稳", "灵动", "飘逸", "坚韧", "柔软",
];

static CHINESE_NOUNS: &[&str] = &[
    "风", "云", "星", "月", "花", "叶", "鸟", "鱼", "猫", "狗", "山", "海", "河", "湖", "树", "草",
    "石", "玉", "金", "银", "梦", "心", "灵", "魂", "光", "影", "音", "色", "香", "味", "书", "画",
    "诗", "歌", "舞", "剑", "琴", "棋", "茶", "酒",
];

static CHINESE_VERBS: &[&str] = &[
    "飞", "游", "跑", "跳", "走", "看", "听", "说", "想", "做", "爱", "恋", "思", "念", "寻", "找",
    "追", "逐", "守", "护",
];

// 英文词库
static ENGLISH_ADJECTIVES: &[&str] = &[
    "brave", "smart", "kind", "cool", "bright", "calm", "warm", "fresh", "wild", "gentle", "bold",
    "swift", "quiet", "loud", "sweet", "sharp", "smooth", "rough", "soft", "hard", "fast", "slow",
    "high", "low", "big", "small", "new", "old", "young", "wise",
];

static ENGLISH_NOUNS: &[&str] = &[
    "wind", "cloud", "star", "moon", "sun", "rain", "snow", "fire", "water", "earth", "bird",
    "fish", "cat", "dog", "lion", "tiger", "wolf", "bear", "eagle", "dragon", "tree", "flower",
    "leaf", "stone", "mountain", "ocean", "river", "lake", "forest", "desert", "dream", "heart",
    "soul", "spirit", "light", "shadow", "sound", "color", "song", "dance",
];

static ENGLISH_VERBS: &[&str] = &[
    "fly", "run", "jump", "walk", "swim", "dance", "sing", "play", "read", "write", "love",
    "dream", "think", "feel", "see", "hear", "touch", "taste", "smell", "know",
];

impl NicknameGenerator {
    /// 生成昵称
    ///
    /// # 参数
    /// - `config`: 生成配置
    ///
    /// # 返回
    /// - 生成的昵称字符串
    ///
    /// # 性能
    /// - 时间复杂度：O(1)
    /// - 空间复杂度：O(1)
    pub fn generate(config: NicknameConfig) -> String {
        let mut rng = rand::thread_rng();

        let mode = match config.mode {
            NicknameMode::Auto => match rng.gen_range(0..3) {
                0 => NicknameMode::ChineseOnly,
                1 => NicknameMode::EnglishOnly,
                _ => NicknameMode::Mixed,
            },
            m => m,
        };

        let nickname = match mode {
            NicknameMode::ChineseOnly => Self::generate_chinese(&config, &mut rng),
            NicknameMode::EnglishOnly => Self::generate_english(&config, &mut rng),
            NicknameMode::Mixed => Self::generate_mixed(&config, &mut rng),
            NicknameMode::Auto => unreachable!(),
        };

        // 确保长度不超过配置的最大长度
        Self::truncate_to_max_length(&nickname, config.max_length)
    }

    /// 生成纯中文昵称
    fn generate_chinese(config: &NicknameConfig, rng: &mut impl Rng) -> String {
        let pattern = rng.gen_range(0..4);

        let nickname = match pattern {
            // 形容词 + 名词
            0 => {
                let adj = CHINESE_ADJECTIVES[rng.gen_range(0..CHINESE_ADJECTIVES.len())];
                let noun = CHINESE_NOUNS[rng.gen_range(0..CHINESE_NOUNS.len())];
                format!("{}{}", adj, noun)
            }
            // 名词 + 动词
            1 => {
                let noun = CHINESE_NOUNS[rng.gen_range(0..CHINESE_NOUNS.len())];
                let verb = CHINESE_VERBS[rng.gen_range(0..CHINESE_VERBS.len())];
                format!("{}{}", noun, verb)
            }
            // 形容词 + 名词 + 动词
            2 => {
                let adj = CHINESE_ADJECTIVES[rng.gen_range(0..CHINESE_ADJECTIVES.len())];
                let noun = CHINESE_NOUNS[rng.gen_range(0..CHINESE_NOUNS.len())];
                let verb = CHINESE_VERBS[rng.gen_range(0..CHINESE_VERBS.len())];
                format!("{}{}{}", adj, noun, verb)
            }
            // 名词 + 名词
            _ => {
                let noun1 = CHINESE_NOUNS[rng.gen_range(0..CHINESE_NOUNS.len())];
                let noun2 = CHINESE_NOUNS[rng.gen_range(0..CHINESE_NOUNS.len())];
                format!("{}{}", noun1, noun2)
            }
        };

        Self::add_uniqueness_suffix(&nickname, config.with_number, rng)
    }

    /// 生成纯英文昵称
    fn generate_english(config: &NicknameConfig, rng: &mut impl Rng) -> String {
        let pattern = rng.gen_range(0..4);

        let nickname = match pattern {
            // Adjective + Noun
            0 => {
                let adj = ENGLISH_ADJECTIVES[rng.gen_range(0..ENGLISH_ADJECTIVES.len())];
                let noun = ENGLISH_NOUNS[rng.gen_range(0..ENGLISH_NOUNS.len())];
                format!("{}{}", Self::capitalize(adj), Self::capitalize(noun))
            }
            // Noun + Verb
            1 => {
                let noun = ENGLISH_NOUNS[rng.gen_range(0..ENGLISH_NOUNS.len())];
                let verb = ENGLISH_VERBS[rng.gen_range(0..ENGLISH_VERBS.len())];
                format!("{}{}", Self::capitalize(noun), Self::capitalize(verb))
            }
            // Adjective + Noun + Verb
            2 => {
                let adj = ENGLISH_ADJECTIVES[rng.gen_range(0..ENGLISH_ADJECTIVES.len())];
                let noun = ENGLISH_NOUNS[rng.gen_range(0..ENGLISH_NOUNS.len())];
                let verb = ENGLISH_VERBS[rng.gen_range(0..ENGLISH_VERBS.len())];
                format!(
                    "{}{}{}",
                    Self::capitalize(adj),
                    Self::capitalize(noun),
                    Self::capitalize(verb)
                )
            }
            // Noun + Noun
            _ => {
                let noun1 = ENGLISH_NOUNS[rng.gen_range(0..ENGLISH_NOUNS.len())];
                let noun2 = ENGLISH_NOUNS[rng.gen_range(0..ENGLISH_NOUNS.len())];
                format!("{}{}", Self::capitalize(noun1), Self::capitalize(noun2))
            }
        };

        Self::add_uniqueness_suffix(&nickname, config.with_number, rng)
    }

    /// 生成中英文混合昵称
    fn generate_mixed(config: &NicknameConfig, rng: &mut impl Rng) -> String {
        let pattern = rng.gen_range(0..4);

        let nickname = match pattern {
            // 中文形容词 + 英文名词
            0 => {
                let adj = CHINESE_ADJECTIVES[rng.gen_range(0..CHINESE_ADJECTIVES.len())];
                let noun = ENGLISH_NOUNS[rng.gen_range(0..ENGLISH_NOUNS.len())];
                format!("{}{}", adj, Self::capitalize(noun))
            }
            // 英文形容词 + 中文名词
            1 => {
                let adj = ENGLISH_ADJECTIVES[rng.gen_range(0..ENGLISH_ADJECTIVES.len())];
                let noun = CHINESE_NOUNS[rng.gen_range(0..CHINESE_NOUNS.len())];
                format!("{}{}", Self::capitalize(adj), noun)
            }
            // 中文名词 + 英文动词
            2 => {
                let noun = CHINESE_NOUNS[rng.gen_range(0..CHINESE_NOUNS.len())];
                let verb = ENGLISH_VERBS[rng.gen_range(0..ENGLISH_VERBS.len())];
                format!("{}{}", noun, Self::capitalize(verb))
            }
            // 英文名词 + 中文动词
            _ => {
                let noun = ENGLISH_NOUNS[rng.gen_range(0..ENGLISH_NOUNS.len())];
                let verb = CHINESE_VERBS[rng.gen_range(0..CHINESE_VERBS.len())];
                format!("{}{}", Self::capitalize(noun), verb)
            }
        };

        Self::add_uniqueness_suffix(&nickname, config.with_number, rng)
    }

    /// 添加唯一性后缀（使用时间戳哈希确保低重复率）
    fn add_uniqueness_suffix(base: &str, with_number: bool, rng: &mut impl Rng) -> String {
        // 使用纳秒级时间戳 + 随机数生成唯一后缀
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let random = rng.gen::<u64>();

        // 使用简单的哈希算法生成短字符串
        let hash = Self::simple_hash(timestamp, random);

        if with_number {
            // 数字只能结尾，使用哈希值的后4位数字
            let num_suffix = (hash % 10000) as u32;
            format!("{}{}", base, num_suffix)
        } else {
            // 使用哈希值生成短字符串后缀（字母+数字混合，但数字在末尾）
            let suffix = Self::hash_to_string(hash);
            format!("{}{}", base, suffix)
        }
    }

    /// 简单哈希函数（快速）
    fn simple_hash(timestamp: u128, random: u64) -> u64 {
        let t = timestamp as u64;
        ((t.wrapping_mul(31)).wrapping_add(random.wrapping_mul(17))) ^ (t >> 32)
    }

    /// 将哈希值转换为短字符串（确保不以数字开头）
    fn hash_to_string(hash: u64) -> String {
        const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
        const DIGITS: &[u8] = b"0123456789";

        let mut result = String::new();
        let mut h = hash;

        // 先添加2-3个字母（确保不以数字开头）
        for _ in 0..3 {
            result.push(CHARS[(h % CHARS.len() as u64) as usize] as char);
            h /= CHARS.len() as u64;
        }

        // 再添加1-2个数字（数字在末尾）
        for _ in 0..2 {
            result.push(DIGITS[(h % DIGITS.len() as u64) as usize] as char);
            h /= DIGITS.len() as u64;
        }

        result
    }

    /// 首字母大写
    fn capitalize(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }

    /// 截断到最大长度
    fn truncate_to_max_length(s: &str, max_length: usize) -> String {
        if s.chars().count() <= max_length {
            return s.to_string();
        }

        // 按字符截断（不是字节）
        s.chars().take(max_length).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_chinese() {
        let config = NicknameConfig {
            mode: NicknameMode::ChineseOnly,
            with_number: false,
            max_length: 16,
        };

        for _ in 0..10 {
            let nickname = NicknameGenerator::generate(config.clone());
            println!("中文昵称: {}", nickname);
            assert!(nickname.chars().count() <= 16);
        }
    }

    #[test]
    fn test_generate_english() {
        let config = NicknameConfig {
            mode: NicknameMode::EnglishOnly,
            with_number: false,
            max_length: 16,
        };

        for _ in 0..10 {
            let nickname = NicknameGenerator::generate(config.clone());
            println!("英文昵称: {}", nickname);
            assert!(nickname.chars().count() <= 16);
            assert!(!nickname.chars().next().unwrap().is_ascii_digit());
        }
    }

    #[test]
    fn test_generate_mixed() {
        let config = NicknameConfig {
            mode: NicknameMode::Mixed,
            with_number: false,
            max_length: 16,
        };

        for _ in 0..10 {
            let nickname = NicknameGenerator::generate(config.clone());
            println!("混合昵称: {}", nickname);
            assert!(nickname.chars().count() <= 16);
        }
    }

    #[test]
    fn test_with_number() {
        let config = NicknameConfig {
            mode: NicknameMode::ChineseOnly,
            with_number: true,
            max_length: 16,
        };

        for _ in 0..10 {
            let nickname = NicknameGenerator::generate(config.clone());
            println!("带数字昵称: {}", nickname);
            assert!(nickname.chars().count() <= 16);
            // 检查数字在末尾
            let last_char = nickname.chars().last().unwrap();
            assert!(last_char.is_ascii_digit());
        }
    }

    #[test]
    fn test_max_length() {
        let config = NicknameConfig {
            mode: NicknameMode::Auto,
            with_number: false,
            max_length: 8,
        };

        for _ in 0..10 {
            let nickname = NicknameGenerator::generate(config.clone());
            println!("限制长度昵称: {}", nickname);
            assert!(nickname.chars().count() <= 8);
        }
    }
}
