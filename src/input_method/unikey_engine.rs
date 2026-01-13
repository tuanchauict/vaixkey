// Vietnamese Input Engine based on Unikey/Uk362 algorithm
// This implementation follows the VietKey class logic from the original Unikey project
// Original copyright: Pham Kim Long (UniKey project)
// Rust port for VaixKey

use std::collections::HashMap;

/// Maximum buffer size for storing typed characters
const KEY_BUFSIZE: usize = 40;
/// Number of characters to maintain when buffer is full
const KEYS_MAINTAIN: usize = 20;
/// Maximum characters after a vowel for tone placement
const MAX_AFTER_VOWEL: usize = 2;
/// Maximum vowel sequence length
const MAX_VOWEL_SEQUENCE: usize = 3;
/// Maximum length for character modification lookup
const MAX_MODIFY_LENGTH: usize = 6;

/// Input method types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMethod {
    Telex,
    Vni,
    Viqr,
}

/// Key categories for Vietnamese input processing
#[derive(Debug, Clone, Copy, PartialEq)]
enum KeyCategory {
    None,
    BreveMark,      // For ă (aw in Telex), ơ, ư (ow, uw)
    ToneMark,       // Tone marks: s, f, r, x, j in Telex
    DoubleKey,      // Double characters: aa, ee, oo, dd
    ShortKey,       // Shortcut keys: [, ], w
    VowelChar,      // Vowel characters
    Separator,      // Word separators (space, punctuation)
    VniDoubleMark,  // VNI specific: 6, 7, 8, 9
    EscapeKey,      // Escape key (\ in VIQR)
    SoftSeparator,  // Soft separators that don't clear buffer
}

/// Character attributes for the DT table
#[derive(Debug, Clone, Copy, Default)]
struct CharAttr {
    vowel_index: u8,      // Index of vowel (0 = not a vowel)
    is_breve: bool,       // Is this a breve mark key (w, W)
    tone_index: u8,       // Tone mark index (1-5 for s,f,r,x,j)
    dbchar_index: u8,     // Double character index (aa, ee, etc.)
    macro_index: u8,      // Macro/shortcut key index
    is_separator: bool,   // Is a word separator
    is_soft_sep: bool,    // Is a soft separator
    vni_double_index: u8, // VNI double mark index
    current_tone: u8,     // Current tone on the character (0 = no tone)
}

/// The main Vietnamese processing engine based on Unikey algorithm
#[derive(Debug)]
pub struct UnikeyEngine {
    // Buffer state
    keys: usize,
    buf: [char; KEY_BUFSIZE],
    lower_case: [bool; KEY_BUFSIZE],
    
    // Processing state
    last_w_converted: bool,
    last_is_escape: bool,
    temp_viet_off: bool,
    
    // Configuration
    input_method: InputMethod,
    vietnamese_mode: bool,
    free_marking: bool,
    tone_next_to_vowel: bool,
    modern_style: bool,
    
    // Output
    keys_pushed: usize,
    backs: usize,
    output_buffer: String,
    
    // Lookup tables
    dt: HashMap<char, CharAttr>,
    
    // Vietnamese character mappings
    // BD[vowel_index][tone_index] = toned character
    // Index 5 = base character without tone
    bd: [[char; 6]; 12],
    
    // BK: Double character results (a->â, e->ê, o->ô, d->đ)
    bk: [char; 8],
    
    // BW: Breve/horn results (a->ă, o->ơ, u->ư)  
    bw: [char; 6],
    
    // BT: Shortcut results
    bt: [char; 4],
}

impl UnikeyEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            keys: 0,
            buf: ['\0'; KEY_BUFSIZE],
            lower_case: [true; KEY_BUFSIZE],
            last_w_converted: false,
            last_is_escape: false,
            temp_viet_off: false,
            input_method: InputMethod::Telex,
            vietnamese_mode: true,
            free_marking: true,
            tone_next_to_vowel: false,
            modern_style: true,
            keys_pushed: 0,
            backs: 0,
            output_buffer: String::new(),
            dt: HashMap::new(),
            bd: [['\0'; 6]; 12],
            bk: ['\0'; 8],
            bw: ['\0'; 6],
            bt: ['\0'; 4],
        };
        engine.init_tables();
        engine
    }

    /// Initialize all lookup tables
    fn init_tables(&mut self) {
        self.init_dt_table();
        self.init_bd_table();
        self.init_bk_table();
        self.init_bw_table();
        self.init_bt_table();
    }

    /// Initialize the DT (character attributes) table
    fn init_dt_table(&mut self) {
        // Vowels with their indices
        // a=1, â=2, ă=3, e=4, ê=5, i=6, o=7, ô=8, ơ=9, u=10, ư=11, y=12
        let vowel_base = [
            ('a', 1), ('A', 1),
            ('e', 4), ('E', 4),
            ('i', 6), ('I', 6),
            ('o', 7), ('O', 7),
            ('u', 10), ('U', 10),
            ('y', 12), ('Y', 12),
        ];
        
        for (c, idx) in vowel_base {
            self.dt.insert(c, CharAttr {
                vowel_index: idx,
                dbchar_index: idx, // Can be doubled
                ..Default::default()
            });
        }

        // Vietnamese vowels with their indices
        let vn_vowels = [
            // â family (index 2)
            ('â', 2, 0), ('ấ', 2, 1), ('ầ', 2, 2), ('ẩ', 2, 3), ('ẫ', 2, 4), ('ậ', 2, 5),
            ('Â', 2, 0), ('Ấ', 2, 1), ('Ầ', 2, 2), ('Ẩ', 2, 3), ('Ẫ', 2, 4), ('Ậ', 2, 5),
            // ă family (index 3)
            ('ă', 3, 0), ('ắ', 3, 1), ('ằ', 3, 2), ('ẳ', 3, 3), ('ẵ', 3, 4), ('ặ', 3, 5),
            ('Ă', 3, 0), ('Ắ', 3, 1), ('Ằ', 3, 2), ('Ẳ', 3, 3), ('Ẵ', 3, 4), ('Ặ', 3, 5),
            // ê family (index 5)
            ('ê', 5, 0), ('ế', 5, 1), ('ề', 5, 2), ('ể', 5, 3), ('ễ', 5, 4), ('ệ', 5, 5),
            ('Ê', 5, 0), ('Ế', 5, 1), ('Ề', 5, 2), ('Ể', 5, 3), ('Ễ', 5, 4), ('Ệ', 5, 5),
            // ô family (index 8)
            ('ô', 8, 0), ('ố', 8, 1), ('ồ', 8, 2), ('ổ', 8, 3), ('ỗ', 8, 4), ('ộ', 8, 5),
            ('Ô', 8, 0), ('Ố', 8, 1), ('Ồ', 8, 2), ('Ổ', 8, 3), ('Ỗ', 8, 4), ('Ộ', 8, 5),
            // ơ family (index 9)
            ('ơ', 9, 0), ('ớ', 9, 1), ('ờ', 9, 2), ('ở', 9, 3), ('ỡ', 9, 4), ('ợ', 9, 5),
            ('Ơ', 9, 0), ('Ớ', 9, 1), ('Ờ', 9, 2), ('Ở', 9, 3), ('Ỡ', 9, 4), ('Ợ', 9, 5),
            // ư family (index 11)
            ('ư', 11, 0), ('ứ', 11, 1), ('ừ', 11, 2), ('ử', 11, 3), ('ữ', 11, 4), ('ự', 11, 5),
            ('Ư', 11, 0), ('Ứ', 11, 1), ('Ừ', 11, 2), ('Ử', 11, 3), ('Ữ', 11, 4), ('Ự', 11, 5),
            // Toned base vowels
            ('á', 1, 1), ('à', 1, 2), ('ả', 1, 3), ('ã', 1, 4), ('ạ', 1, 5),
            ('Á', 1, 1), ('À', 1, 2), ('Ả', 1, 3), ('Ã', 1, 4), ('Ạ', 1, 5),
            ('é', 4, 1), ('è', 4, 2), ('ẻ', 4, 3), ('ẽ', 4, 4), ('ẹ', 4, 5),
            ('É', 4, 1), ('È', 4, 2), ('Ẻ', 4, 3), ('Ẽ', 4, 4), ('Ẹ', 4, 5),
            ('í', 6, 1), ('ì', 6, 2), ('ỉ', 6, 3), ('ĩ', 6, 4), ('ị', 6, 5),
            ('Í', 6, 1), ('Ì', 6, 2), ('Ỉ', 6, 3), ('Ĩ', 6, 4), ('Ị', 6, 5),
            ('ó', 7, 1), ('ò', 7, 2), ('ỏ', 7, 3), ('õ', 7, 4), ('ọ', 7, 5),
            ('Ó', 7, 1), ('Ò', 7, 2), ('Ỏ', 7, 3), ('Õ', 7, 4), ('Ọ', 7, 5),
            ('ú', 10, 1), ('ù', 10, 2), ('ủ', 10, 3), ('ũ', 10, 4), ('ụ', 10, 5),
            ('Ú', 10, 1), ('Ù', 10, 2), ('Ủ', 10, 3), ('Ũ', 10, 4), ('Ụ', 10, 5),
            ('ý', 12, 1), ('ỳ', 12, 2), ('ỷ', 12, 3), ('ỹ', 12, 4), ('ỵ', 12, 5),
            ('Ý', 12, 1), ('Ỳ', 12, 2), ('Ỷ', 12, 3), ('Ỹ', 12, 4), ('Ỵ', 12, 5),
        ];
        
        for (c, vowel_idx, tone_idx) in vn_vowels {
            self.dt.insert(c, CharAttr {
                vowel_index: vowel_idx,
                current_tone: tone_idx,
                ..Default::default()
            });
        }

        // Telex tone keys
        let tone_keys = [
            ('s', 1), ('S', 1), // sắc
            ('f', 2), ('F', 2), // huyền
            ('r', 3), ('R', 3), // hỏi
            ('x', 4), ('X', 4), // ngã
            ('j', 5), ('J', 5), // nặng
        ];
        
        for (c, tone) in tone_keys {
            self.dt.insert(c, CharAttr {
                tone_index: tone,
                ..Default::default()
            });
        }

        // Breve/horn keys (w for ă, ơ, ư)
        self.dt.insert('w', CharAttr { is_breve: true, macro_index: 1, ..Default::default() });
        self.dt.insert('W', CharAttr { is_breve: true, macro_index: 1, ..Default::default() });

        // Double character keys (Telex)
        // a, d, e, o can be doubled
        if let Some(attr) = self.dt.get_mut(&'a') { attr.dbchar_index = 1; }
        if let Some(attr) = self.dt.get_mut(&'A') { attr.dbchar_index = 1; }
        self.dt.insert('d', CharAttr { dbchar_index: 2, ..Default::default() });
        self.dt.insert('D', CharAttr { dbchar_index: 2, ..Default::default() });
        if let Some(attr) = self.dt.get_mut(&'e') { attr.dbchar_index = 3; }
        if let Some(attr) = self.dt.get_mut(&'E') { attr.dbchar_index = 3; }
        if let Some(attr) = self.dt.get_mut(&'o') { attr.dbchar_index = 4; }
        if let Some(attr) = self.dt.get_mut(&'O') { attr.dbchar_index = 4; }

        // Separators
        let separators = [' ', '\n', '\r', '\t', '.', ',', ';', ':', '!', '?', 
                         '(', ')', '[', ']', '{', '}', '<', '>', '/', '\\',
                         '"', '\'', '-', '_', '+', '=', '@', '#', '$', '%',
                         '^', '&', '*', '|', '`', '~', '0', '1', '2', '3',
                         '4', '5', '6', '7', '8', '9'];
        
        for c in separators {
            self.dt.insert(c, CharAttr { is_separator: true, ..Default::default() });
        }

        // đ/Đ
        self.dt.insert('đ', CharAttr { dbchar_index: 2, ..Default::default() });
        self.dt.insert('Đ', CharAttr { dbchar_index: 2, ..Default::default() });
    }

    /// Initialize BD table: vowel_index -> [acute, grave, hook, tilde, dot, base]
    fn init_bd_table(&mut self) {
        // a family (index 0, corresponds to vowel_index 1)
        self.bd[0] = ['á', 'à', 'ả', 'ã', 'ạ', 'a'];
        // â family (index 1, corresponds to vowel_index 2)
        self.bd[1] = ['ấ', 'ầ', 'ẩ', 'ẫ', 'ậ', 'â'];
        // ă family (index 2, corresponds to vowel_index 3)
        self.bd[2] = ['ắ', 'ằ', 'ẳ', 'ẵ', 'ặ', 'ă'];
        // e family (index 3, corresponds to vowel_index 4)
        self.bd[3] = ['é', 'è', 'ẻ', 'ẽ', 'ẹ', 'e'];
        // ê family (index 4, corresponds to vowel_index 5)
        self.bd[4] = ['ế', 'ề', 'ể', 'ễ', 'ệ', 'ê'];
        // i family (index 5, corresponds to vowel_index 6)
        self.bd[5] = ['í', 'ì', 'ỉ', 'ĩ', 'ị', 'i'];
        // o family (index 6, corresponds to vowel_index 7)
        self.bd[6] = ['ó', 'ò', 'ỏ', 'õ', 'ọ', 'o'];
        // ô family (index 7, corresponds to vowel_index 8)
        self.bd[7] = ['ố', 'ồ', 'ổ', 'ỗ', 'ộ', 'ô'];
        // ơ family (index 8, corresponds to vowel_index 9)
        self.bd[8] = ['ớ', 'ờ', 'ở', 'ỡ', 'ợ', 'ơ'];
        // u family (index 9, corresponds to vowel_index 10)
        self.bd[9] = ['ú', 'ù', 'ủ', 'ũ', 'ụ', 'u'];
        // ư family (index 10, corresponds to vowel_index 11)
        self.bd[10] = ['ứ', 'ừ', 'ử', 'ữ', 'ự', 'ư'];
        // y family (index 11, corresponds to vowel_index 12)
        self.bd[11] = ['ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ', 'y'];
    }

    /// Initialize BK table: double character results
    /// Index: 0=d->đ, 1=D->Đ, 2=a->â, 3=A->Â, 4=e->ê, 5=E->Ê, 6=o->ô, 7=O->Ô
    fn init_bk_table(&mut self) {
        self.bk = ['đ', 'Đ', 'â', 'Â', 'ê', 'Ê', 'ô', 'Ô'];
    }

    /// Initialize BW table: breve/horn results
    /// Index: 0=a->ă, 1=A->Ă, 2=o->ơ, 3=O->Ơ, 4=u->ư, 5=U->Ư
    fn init_bw_table(&mut self) {
        self.bw = ['ă', 'Ă', 'ơ', 'Ơ', 'ư', 'Ư'];
    }

    /// Initialize BT table: shortcut results
    fn init_bt_table(&mut self) {
        self.bt = ['ơ', 'Ơ', 'ư', 'Ư'];
    }

    /// Clear the buffer
    pub fn clear_buf(&mut self) {
        self.keys = 0;
        self.last_w_converted = false;
        self.last_is_escape = false;
        self.temp_viet_off = false;
        self.output_buffer.clear();
    }

    /// Get key category based on input method
    fn key_category(&self, c: char) -> KeyCategory {
        let attr = self.dt.get(&c).copied().unwrap_or_default();

        if attr.is_breve && self.input_method == InputMethod::Telex {
            return KeyCategory::BreveMark;
        }

        if attr.tone_index > 0 && self.input_method == InputMethod::Telex {
            return KeyCategory::ToneMark;
        }

        if attr.dbchar_index > 0 && self.input_method == InputMethod::Telex {
            return KeyCategory::DoubleKey;
        }

        if attr.macro_index > 0 {
            return KeyCategory::ShortKey;
        }

        if attr.is_separator {
            return KeyCategory::Separator;
        }

        if attr.vni_double_index > 0 && self.input_method == InputMethod::Vni {
            return KeyCategory::VniDoubleMark;
        }

        if c == '\\' && self.input_method == InputMethod::Viqr {
            return KeyCategory::EscapeKey;
        }

        if attr.is_soft_sep {
            return KeyCategory::SoftSeparator;
        }

        KeyCategory::None
    }

    /// Put a character into the buffer
    fn put_char(&mut self, c: char, is_lower: bool) {
        if self.keys >= KEY_BUFSIZE {
            self.throw_buf();
        }
        self.lower_case[self.keys] = is_lower;
        self.buf[self.keys] = c;
        self.keys += 1;
    }

    /// Throw away old buffer contents when full
    fn throw_buf(&mut self) {
        if self.keys > KEYS_MAINTAIN {
            let start = self.keys - KEYS_MAINTAIN;
            for i in 0..KEYS_MAINTAIN {
                self.buf[i] = self.buf[start + i];
                self.lower_case[i] = self.lower_case[start + i];
            }
            self.keys = KEYS_MAINTAIN;
        }
    }

    /// Process a keypress and return the result
    pub fn process(&mut self, c: char) -> ProcessResult {
        self.keys_pushed = 0;
        self.backs = 0;
        self.output_buffer.clear();

        let is_lower = c.is_lowercase();
        let c_lower = c.to_lowercase().next().unwrap_or(c);
        
        if !self.vietnamese_mode {
            self.put_char(c, is_lower);
            return ProcessResult::PassThrough(c);
        }

        if self.temp_viet_off {
            if !c.is_alphabetic() {
                self.temp_viet_off = false;
            }
            let category = self.key_category(c);
            if category == KeyCategory::Separator {
                if c == '\x08' { // Backspace
                    self.process_backspace();
                } else {
                    self.clear_buf();
                }
            } else {
                self.put_char(c, is_lower);
            }
            return ProcessResult::PassThrough(c);
        }

        let category = self.key_category(c);

        match category {
            KeyCategory::BreveMark => {
                if self.input_method == InputMethod::Telex && 
                   self.last_w_converted && 
                   (c == 'w' || c == 'W') {
                    self.short_key(c, is_lower);
                } else {
                    self.put_breve_mark(c, is_lower);
                    if self.input_method == InputMethod::Telex && 
                       self.keys_pushed == 0 && 
                       self.backs == 0 && 
                       (c == 'w' || c == 'W') {
                        self.short_key(c, is_lower);
                        self.last_w_converted = true;
                    }
                }
            }
            KeyCategory::DoubleKey => {
                self.double_char(c, is_lower);
            }
            KeyCategory::ToneMark => {
                self.put_tone_mark(c, is_lower);
            }
            KeyCategory::ShortKey => {
                self.short_key(c, is_lower);
            }
            KeyCategory::Separator => {
                if c == '\x08' { // Backspace
                    self.process_backspace();
                } else {
                    self.clear_buf();
                }
                self.last_w_converted = false;
                return ProcessResult::PassThrough(c);
            }
            _ => {
                // Regular character
                if category != KeyCategory::BreveMark {
                    self.last_w_converted = false;
                }
            }
        }

        if self.keys_pushed == 0 && self.backs == 0 {
            self.put_char(c, is_lower);
            return ProcessResult::PassThrough(c);
        }

        if self.backs > 0 {
            ProcessResult::Replace {
                backspaces: self.backs,
                text: self.output_buffer.clone(),
            }
        } else {
            ProcessResult::Output(self.output_buffer.clone())
        }
    }

    /// Process backspace
    fn process_backspace(&mut self) {
        if self.keys > 0 {
            self.keys -= 1;
            self.backs = 1;
        }
    }

    /// Put a breve/horn mark (w key in Telex)
    fn put_breve_mark(&mut self, c: char, is_lower: bool) {
        if self.keys == 0 {
            return;
        }

        // Find the vowel to apply the mark to
        let mut i = self.keys as i32 - 1;
        let left_most = if self.free_marking { 0 } else { self.keys as i32 - 1 };
        let left_most = left_most.max(self.keys as i32 - MAX_MODIFY_LENGTH as i32);

        while i >= left_most {
            let buf_char = self.buf[i as usize];
            let attr = self.dt.get(&buf_char).copied().unwrap_or_default();
            
            // Check if this is a vowel that can receive the breve/horn
            if attr.vowel_index > 0 {
                let base_vowel = self.get_base_vowel(buf_char);
                let target_char = match base_vowel.to_lowercase().next().unwrap_or(base_vowel) {
                    'a' => if is_lower { 'ă' } else { 'Ă' },
                    'o' => if is_lower { 'ơ' } else { 'Ơ' },
                    'u' => if is_lower { 'ư' } else { 'Ư' },
                    _ => {
                        i -= 1;
                        continue;
                    }
                };

                // Check for duplicate (undo)
                if self.buf[i as usize] == target_char {
                    // Revert to original
                    self.backs = (self.keys - i as usize) as usize;
                    self.buf[i as usize] = base_vowel;
                    self.rebuild_output(i as usize);
                    self.output_buffer.push(c);
                    self.put_char(c, is_lower);
                    self.temp_viet_off = true;
                    return;
                }

                // Apply the transformation
                let current_tone = attr.current_tone;
                let new_char = if current_tone > 0 {
                    self.apply_tone_to_base(target_char, current_tone)
                } else {
                    target_char
                };

                self.backs = (self.keys - i as usize) as usize;
                self.buf[i as usize] = new_char;
                self.rebuild_output(i as usize);
                self.keys_pushed = self.output_buffer.len();
                return;
            }

            if attr.is_separator || attr.is_soft_sep {
                break;
            }
            i -= 1;
        }
    }

    /// Process double character (aa, ee, oo, dd)
    fn double_char(&mut self, c: char, is_lower: bool) {
        if self.keys == 0 {
            return;
        }

        let last_char = self.buf[self.keys - 1];
        let last_lower = last_char.to_lowercase().next().unwrap_or(last_char);
        let c_lower = c.to_lowercase().next().unwrap_or(c);

        // Check if this is a valid double character
        if last_lower != c_lower {
            return;
        }

        let target = match c_lower {
            'a' => if is_lower { 'â' } else { 'Â' },
            'e' => if is_lower { 'ê' } else { 'Ê' },
            'o' => if is_lower { 'ô' } else { 'Ô' },
            'd' => if is_lower { 'đ' } else { 'Đ' },
            _ => return,
        };

        // Check for undo (triple char)
        let last_attr = self.dt.get(&last_char).copied().unwrap_or_default();
        if last_attr.vowel_index > 0 {
            let base = self.get_base_vowel(last_char);
            if base.to_lowercase().next().unwrap_or(base) != c_lower {
                // Already transformed, undo
                self.backs = 1;
                let original = if is_lower { c_lower } else { c_lower.to_uppercase().next().unwrap_or(c) };
                self.buf[self.keys - 1] = original;
                self.output_buffer.push(original);
                self.output_buffer.push(c);
                self.put_char(c, is_lower);
                self.temp_viet_off = true;
                self.keys_pushed = 2;
                return;
            }
        }

        // Check if the last char is already the target (undo case)
        let last_base = self.get_base_vowel(last_char);
        if last_base.to_lowercase().next() == target.to_lowercase().next() {
            // Already transformed, undo by outputting original + new char
            self.backs = 1;
            let original = if self.lower_case[self.keys - 1] { c_lower } else { c_lower.to_uppercase().next().unwrap_or(c) };
            self.buf[self.keys - 1] = original;
            self.output_buffer.push(original);
            self.output_buffer.push(c);
            self.put_char(c, is_lower);
            self.temp_viet_off = true;
            self.keys_pushed = 2;
            return;
        }

        // Apply transformation
        let last_attr = self.dt.get(&last_char).copied().unwrap_or_default();
        let new_char = if last_attr.current_tone > 0 {
            self.apply_tone_to_base(target, last_attr.current_tone)
        } else {
            target
        };

        self.backs = 1;
        self.buf[self.keys - 1] = new_char;
        self.output_buffer.push(new_char);
        self.keys_pushed = 1;
    }

    /// Put a tone mark (s, f, r, x, j in Telex)
    fn put_tone_mark(&mut self, c: char, is_lower: bool) {
        if self.keys == 0 {
            return;
        }

        let tone_index = match c.to_lowercase().next().unwrap_or(c) {
            's' => 1, // acute (sắc)
            'f' => 2, // grave (huyền)
            'r' => 3, // hook (hỏi)
            'x' => 4, // tilde (ngã)
            'j' => 5, // dot (nặng)
            _ => return,
        };

        // Find the vowel to apply the tone to
        let mut i = self.keys as i32 - 1;
        let left_most = if self.tone_next_to_vowel { i } else { 0 };
        let left_most = left_most.max(self.keys as i32 - 1 - MAX_AFTER_VOWEL as i32);

        // Find the first vowel from the right
        while i >= left_most {
            let attr = self.dt.get(&self.buf[i as usize]).copied().unwrap_or_default();
            if attr.is_separator || attr.is_soft_sep || attr.vowel_index > 0 {
                break;
            }
            i -= 1;
        }

        if i < left_most {
            return;
        }

        let attr = self.dt.get(&self.buf[i as usize]).copied().unwrap_or_default();
        if attr.vowel_index == 0 {
            return;
        }

        // Find the sequence of consecutive vowels
        let end_pos = i;
        let left_most = if self.tone_next_to_vowel { i } else { 0 };
        let left_most = left_most.max(end_pos - MAX_VOWEL_SEQUENCE as i32 + 1);

        while i >= left_most {
            let attr = self.dt.get(&self.buf[i as usize]).copied().unwrap_or_default();
            if attr.vowel_index == 0 {
                break;
            }
            // Stop if we hit a toned vowel (to replace the tone)
            let base_char = self.buf[i as usize];
            if base_char.is_alphabetic() && !base_char.is_ascii_alphabetic() {
                break;
            }
            i -= 1;
        }

        // Determine which vowel to apply the tone to
        let vowel_seq_len = (end_pos - i) as usize;
        let target_pos = match vowel_seq_len {
            2 => {
                // Check for special cases: oa, oe, uy -> tone on second vowel
                if self.modern_style {
                    let v1 = self.buf[(end_pos - 1) as usize].to_lowercase().next().unwrap_or(' ');
                    let v2 = self.buf[end_pos as usize].to_lowercase().next().unwrap_or(' ');
                    if (v1 == 'o' && v2 == 'a') || (v1 == 'o' && v2 == 'e') || (v1 == 'u' && v2 == 'y') {
                        end_pos as usize
                    } else {
                        // Check for qu, gi patterns
                        if i >= 0 {
                            let prev = self.buf[i as usize].to_uppercase().next().unwrap_or(' ');
                            if prev == 'Q' || (prev == 'G' && i + 1 < self.keys as i32 && 
                                self.buf[(i + 1) as usize].to_uppercase().next().unwrap_or(' ') == 'I') {
                                end_pos as usize
                            } else if self.keys as i32 > end_pos + 1 {
                                end_pos as usize
                            } else {
                                (end_pos - 1) as usize
                            }
                        } else {
                            (end_pos - 1) as usize
                        }
                    }
                } else {
                    (end_pos - 1) as usize
                }
            }
            3 => (end_pos - 1) as usize,
            _ => end_pos as usize,
        };

        // Get the vowel and apply the tone
        let vowel_char = self.buf[target_pos];
        let vowel_attr = self.dt.get(&vowel_char).copied().unwrap_or_default();
        let vowel_idx = vowel_attr.vowel_index as usize;
        
        if vowel_idx == 0 || vowel_idx > 12 {
            return;
        }

        // Check for duplicate tone (undo)
        let current_tone = vowel_attr.current_tone;
        if current_tone == tone_index {
            // Remove the tone
            let base = self.bd[vowel_idx - 1][5];
            let new_char = if vowel_char.is_uppercase() {
                base.to_uppercase().next().unwrap_or(base)
            } else {
                base
            };
            self.backs = self.keys - target_pos;
            self.buf[target_pos] = new_char;
            self.rebuild_output(target_pos);
            self.output_buffer.push(c);
            self.put_char(c, is_lower);
            self.temp_viet_off = true;
            return;
        }

        // Apply the tone
        let base = self.get_base_vowel(vowel_char);
        let base_attr = self.dt.get(&base).copied().unwrap_or_default();
        let base_idx = if base_attr.vowel_index > 0 { base_attr.vowel_index as usize } else { vowel_idx };
        
        if base_idx == 0 || base_idx > 12 {
            return;
        }

        let new_char = self.bd[base_idx - 1][tone_index as usize - 1];
        let new_char = if vowel_char.is_uppercase() {
            new_char.to_uppercase().next().unwrap_or(new_char)
        } else {
            new_char
        };

        self.backs = self.keys - target_pos;
        self.buf[target_pos] = new_char;
        self.rebuild_output(target_pos);
        self.keys_pushed = self.output_buffer.len();
    }

    /// Process a shortcut key
    fn short_key(&mut self, c: char, is_lower: bool) {
        // For 'w' alone, output ư
        let new_char = match c.to_lowercase().next().unwrap_or(c) {
            'w' => if is_lower { 'ư' } else { 'Ư' },
            '[' => 'ơ',
            ']' => 'Ơ',
            _ => return,
        };

        // Check for duplicate (undo)
        if self.keys > 0 && self.buf[self.keys - 1] == new_char {
            self.backs = 1;
            self.buf[self.keys - 1] = c;
            self.output_buffer.push(c);
            self.temp_viet_off = true;
            self.keys_pushed = 1;
            return;
        }

        self.output_buffer.push(new_char);
        self.put_char(new_char, is_lower);
        self.keys_pushed = 1;
    }

    /// Get the base vowel (without tone) for a character
    fn get_base_vowel(&self, c: char) -> char {
        let attr = self.dt.get(&c).copied().unwrap_or_default();
        if attr.vowel_index == 0 || attr.vowel_index > 12 {
            return c;
        }
        
        let base = self.bd[attr.vowel_index as usize - 1][5];
        if c.is_uppercase() {
            base.to_uppercase().next().unwrap_or(base)
        } else {
            base
        }
    }

    /// Apply a tone to a base vowel
    fn apply_tone_to_base(&self, base: char, tone: u8) -> char {
        let attr = self.dt.get(&base).copied().unwrap_or_default();
        if attr.vowel_index == 0 || attr.vowel_index > 12 || tone == 0 || tone > 5 {
            return base;
        }

        let toned = self.bd[attr.vowel_index as usize - 1][tone as usize - 1];
        if base.is_uppercase() {
            toned.to_uppercase().next().unwrap_or(toned)
        } else {
            toned
        }
    }

    /// Rebuild output from a position in the buffer
    fn rebuild_output(&mut self, from_pos: usize) {
        self.output_buffer.clear();
        for i in from_pos..self.keys {
            self.output_buffer.push(self.buf[i]);
        }
    }

    // Public API methods
    
    pub fn set_input_method(&mut self, method: InputMethod) {
        self.input_method = method;
    }

    pub fn set_vietnamese_mode(&mut self, enabled: bool) {
        self.vietnamese_mode = enabled;
        if !enabled {
            self.clear_buf();
        }
    }

    pub fn toggle_vietnamese_mode(&mut self) {
        self.set_vietnamese_mode(!self.vietnamese_mode);
    }

    pub fn is_vietnamese_mode(&self) -> bool {
        self.vietnamese_mode
    }

    pub fn get_buffer(&self) -> String {
        self.buf[..self.keys].iter().collect()
    }

    pub fn set_free_marking(&mut self, enabled: bool) {
        self.free_marking = enabled;
    }

    pub fn set_modern_style(&mut self, enabled: bool) {
        self.modern_style = enabled;
    }
}

impl Default for UnikeyEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of processing a keypress
#[derive(Debug, Clone)]
pub enum ProcessResult {
    /// Pass the character through unchanged
    PassThrough(char),
    /// Output new text (append to current text)
    Output(String),
    /// Replace text: delete backspaces characters, then output text
    Replace {
        backspaces: usize,
        text: String,
    },
}
