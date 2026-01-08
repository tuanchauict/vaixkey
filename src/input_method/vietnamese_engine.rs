use std::collections::HashMap;

#[derive(Debug)]
pub struct VietnameseEngine {
    telex_map: HashMap<&'static str, &'static str>,
    vni_map: HashMap<&'static str, &'static str>,
}

impl VietnameseEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            telex_map: HashMap::new(),
            vni_map: HashMap::new(),
        };
        engine.init_telex_mapping();
        engine.init_vni_mapping();
        engine
    }

    fn init_telex_mapping(&mut self) {
        // Vowels with tones
        self.telex_map.insert("a", "a");
        self.telex_map.insert("aa", "â");
        self.telex_map.insert("aw", "ă");
        self.telex_map.insert("e", "e");
        self.telex_map.insert("ee", "ê");
        self.telex_map.insert("i", "i");
        self.telex_map.insert("o", "o");
        self.telex_map.insert("oo", "ô");
        self.telex_map.insert("ow", "ơ");
        self.telex_map.insert("u", "u");
        self.telex_map.insert("uw", "ư");
        self.telex_map.insert("y", "y");

        // Consonants
        self.telex_map.insert("d", "d");
        self.telex_map.insert("dd", "đ");

        // Tone marks - these will be applied to the last vowel
        // We'll handle tones separately in the processing logic
    }

    fn init_vni_mapping(&mut self) {
        // VNI mappings (numbers for tones and special characters)
        self.vni_map.insert("a6", "ă");
        self.vni_map.insert("a8", "â");
        self.vni_map.insert("e6", "ê");
        self.vni_map.insert("o6", "ô");
        self.vni_map.insert("o7", "ơ");
        self.vni_map.insert("u7", "ư");
        self.vni_map.insert("d9", "đ");
    }

    pub fn process_telex(&self, input: &str) -> Option<String> {
        if input.is_empty() {
            return None;
        }

        let mut result = String::new();
        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let char = chars[i];

            // Check for double characters first
            if i + 1 < chars.len() {
                let double_char = format!("{}{}", char, chars[i + 1]);
                match double_char.as_str() {
                    "aa" => {
                        result.push('â');
                        i += 2;
                        continue;
                    }
                    "aw" => {
                        result.push('ă');
                        i += 2;
                        continue;
                    }
                    "ee" => {
                        result.push('ê');
                        i += 2;
                        continue;
                    }
                    "oo" => {
                        result.push('ô');
                        i += 2;
                        continue;
                    }
                    "ow" => {
                        result.push('ơ');
                        i += 2;
                        continue;
                    }
                    "uw" => {
                        result.push('ư');
                        i += 2;
                        continue;
                    }
                    "dd" => {
                        result.push('đ');
                        i += 2;
                        continue;
                    }
                    _ => {}
                }
            }

            // Handle tone marks
            match char {
                's' | 'f' | 'r' | 'x' | 'j' => {
                    // These are tone markers, apply to the last vowel in result
                    if let Some(last_vowel_pos) = self.find_last_vowel_position(&result) {
                        let tone = match char {
                            's' => ToneMark::Acute,      // á, é, í, ó, ú, ý
                            'f' => ToneMark::Grave,      // à, è, ì, ò, ù, ỳ
                            'r' => ToneMark::Hook,       // ả, ẻ, ỉ, ỏ, ủ, ỷ
                            'x' => ToneMark::Tilde,      // ã, ẽ, ĩ, õ, ũ, ỹ
                            'j' => ToneMark::Dot,        // ạ, ẹ, ị, ọ, ụ, ỵ
                            _ => unreachable!(),
                        };

                        let new_result = self.apply_tone_to_position(&result, last_vowel_pos, tone);
                        return Some(new_result);
                    } else {
                        // No vowel to apply tone to, add the character as-is
                        result.push(char);
                    }
                }
                _ => {
                    result.push(char);
                }
            }
            i += 1;
        }

        Some(result)
    }

    pub fn process_vni(&self, input: &str) -> Option<String> {
        // Simple VNI implementation
        let mut result = input.to_string();

        for (vni, vietnamese) in &self.vni_map {
            result = result.replace(vni, vietnamese);
        }

        // Handle tone numbers
        result = self.apply_vni_tones(&result);

        Some(result)
    }

    pub fn process_simple_telex(&self, input: &str) -> Option<String> {
        // Simplified version of Telex without complex tone handling
        self.process_telex(input)
    }

    fn find_last_vowel_position(&self, text: &str) -> Option<usize> {
        let vowels = "aeiouâăêôơưyAEIOUÂĂÊÔƠƯY";
        text.char_indices()
            .rev()
            .find(|(_, c)| vowels.contains(*c))
            .map(|(pos, _)| pos)
    }

    fn apply_tone_to_position(&self, text: &str, pos: usize, tone: ToneMark) -> String {
        let mut chars: Vec<char> = text.chars().collect();
        if pos < chars.len() {
            let original_char = chars[pos];
            if let Some(toned_char) = self.apply_tone_to_char(original_char, tone) {
                chars[pos] = toned_char;
            }
        }
        chars.into_iter().collect()
    }

    fn apply_tone_to_char(&self, c: char, tone: ToneMark) -> Option<char> {
        match (c, tone) {
            // Acute tone (sắc)
            ('a', ToneMark::Acute) => Some('á'),
            ('ă', ToneMark::Acute) => Some('ắ'),
            ('â', ToneMark::Acute) => Some('ấ'),
            ('e', ToneMark::Acute) => Some('é'),
            ('ê', ToneMark::Acute) => Some('ế'),
            ('i', ToneMark::Acute) => Some('í'),
            ('o', ToneMark::Acute) => Some('ó'),
            ('ô', ToneMark::Acute) => Some('ố'),
            ('ơ', ToneMark::Acute) => Some('ớ'),
            ('u', ToneMark::Acute) => Some('ú'),
            ('ư', ToneMark::Acute) => Some('ứ'),
            ('y', ToneMark::Acute) => Some('ý'),

            // Grave tone (huyền)
            ('a', ToneMark::Grave) => Some('à'),
            ('ă', ToneMark::Grave) => Some('ằ'),
            ('â', ToneMark::Grave) => Some('ầ'),
            ('e', ToneMark::Grave) => Some('è'),
            ('ê', ToneMark::Grave) => Some('ề'),
            ('i', ToneMark::Grave) => Some('ì'),
            ('o', ToneMark::Grave) => Some('ò'),
            ('ô', ToneMark::Grave) => Some('ồ'),
            ('ơ', ToneMark::Grave) => Some('ờ'),
            ('u', ToneMark::Grave) => Some('ù'),
            ('ư', ToneMark::Grave) => Some('ừ'),
            ('y', ToneMark::Grave) => Some('ỳ'),

            // Hook tone (hỏi)
            ('a', ToneMark::Hook) => Some('ả'),
            ('ă', ToneMark::Hook) => Some('ẳ'),
            ('â', ToneMark::Hook) => Some('ẩ'),
            ('e', ToneMark::Hook) => Some('ẻ'),
            ('ê', ToneMark::Hook) => Some('ể'),
            ('i', ToneMark::Hook) => Some('ỉ'),
            ('o', ToneMark::Hook) => Some('ỏ'),
            ('ô', ToneMark::Hook) => Some('ổ'),
            ('ơ', ToneMark::Hook) => Some('ở'),
            ('u', ToneMark::Hook) => Some('ủ'),
            ('ư', ToneMark::Hook) => Some('ử'),
            ('y', ToneMark::Hook) => Some('ỷ'),

            // Tilde tone (ngã)
            ('a', ToneMark::Tilde) => Some('ã'),
            ('ă', ToneMark::Tilde) => Some('ẵ'),
            ('â', ToneMark::Tilde) => Some('ẫ'),
            ('e', ToneMark::Tilde) => Some('ẽ'),
            ('ê', ToneMark::Tilde) => Some('ễ'),
            ('i', ToneMark::Tilde) => Some('ĩ'),
            ('o', ToneMark::Tilde) => Some('õ'),
            ('ô', ToneMark::Tilde) => Some('ỗ'),
            ('ơ', ToneMark::Tilde) => Some('ỡ'),
            ('u', ToneMark::Tilde) => Some('ũ'),
            ('ư', ToneMark::Tilde) => Some('ữ'),
            ('y', ToneMark::Tilde) => Some('ỹ'),

            // Dot tone (nặng)
            ('a', ToneMark::Dot) => Some('ạ'),
            ('ă', ToneMark::Dot) => Some('ặ'),
            ('â', ToneMark::Dot) => Some('ậ'),
            ('e', ToneMark::Dot) => Some('ẹ'),
            ('ê', ToneMark::Dot) => Some('ệ'),
            ('i', ToneMark::Dot) => Some('ị'),
            ('o', ToneMark::Dot) => Some('ọ'),
            ('ô', ToneMark::Dot) => Some('ộ'),
            ('ơ', ToneMark::Dot) => Some('ợ'),
            ('u', ToneMark::Dot) => Some('ụ'),
            ('ư', ToneMark::Dot) => Some('ự'),
            ('y', ToneMark::Dot) => Some('ỵ'),

            _ => None,
        }
    }

    fn apply_vni_tones(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Apply VNI tone numbers (1-5)
        // This is a simplified implementation
        result = result.replace("1", ""); // Tone 1 (no mark)
        // Add more VNI tone processing here

        result
    }
}

#[derive(Debug, Clone, Copy)]
enum ToneMark {
    Acute,  // sắc (/)
    Grave,  // huyền (\)
    Hook,   // hỏi (?)
    Tilde,  // ngã (~)
    Dot,    // nặng (.)
}