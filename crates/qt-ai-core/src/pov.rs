//! Đoán ngôi kể từ raw: tỉ lệ đoạn có `我` ở lời kể (bỏ phần trong ngoặc thoại và ý nghĩ có dẫn `心想：我…`).
//! Đo trên thư viện 137 truyện (2026-11-21): truyện kể ngôi ba cao nhất 10,6%, 124 truyện dưới 4%; truyện nhân vật
//! chính tự kể 63% — ngưỡng 30% tách chắc. Dùng để AI điền đặt sẵn ô Ngôi kể và để Hồ sơ truyện cảnh báo khi ô lệch raw.

use crate::story::{GenreSetting, StoryPov};
use crate::story_fs::{list_raw_chapter_ids, read_raw_chapter, StoryPaths};
use crate::Result;
use regex::Regex;
use std::sync::LazyLock;

/// Số chương rải đều trong `raw/` đem đếm.
pub const SAMPLE_CHAPTERS: usize = 60;
/// Từ tỉ lệ này trở lên là kể ngôi một.
pub const FIRST_PERSON_RATIO: f64 = 0.30;
/// Ít đoạn hơn thì không đoán.
pub const MIN_PARAGRAPHS: usize = 30;

static QUOTE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"“[^”]*”|「[^」]*」|『[^』]*』|"[^"]*""#).unwrap());
static THOUGHT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:心想|心道|暗道|想道|心里想|暗想|心中想|自语|喃喃)[：:，,]?.*$").unwrap());

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PovGuess {
    pub chapters: usize,
    pub paragraphs: usize,
    /// Số đoạn có `我` ở lời kể.
    pub first_person: usize,
}

impl PovGuess {
    pub fn ratio(&self) -> f64 {
        if self.paragraphs == 0 {
            0.0
        } else {
            self.first_person as f64 / self.paragraphs as f64
        }
    }

    /// `None` khi raw quá ít để đoán.
    pub fn is_first_person(&self) -> Option<bool> {
        (self.paragraphs >= MIN_PARAGRAPHS).then(|| self.ratio() >= FIRST_PERSON_RATIO)
    }

    /// Ngôi kể gợi ý theo bối cảnh: ngôi một ở truyện hiện đại xưng `tôi` (user chốt 2026-11-21), bối cảnh khác `ta`.
    pub fn suggested(&self, setting: GenreSetting) -> Option<StoryPov> {
        self.is_first_person().map(|first| match (first, setting) {
            (false, _) => StoryPov::Third,
            (true, GenreSetting::Modern) => StoryPov::FirstToi,
            (true, _) => StoryPov::FirstTa,
        })
    }
}

static NARRATOR_WORD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)ngôi (?:thứ )?(?:nhất|một|1)|người kể|lời kể|lời dẫn|dẫn truyện|kể chuyện|tự sự").unwrap()
});
static GROUP_PRONOUN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\b(?:người|chúng|bọn) (?:ta|tôi)\b").unwrap());
static SELF_PRONOUN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\b(?:ta|tôi)\b").unwrap());

/// Dòng style (voice / tone rule) quy định người kể tự xưng `ta` / `tôi` — việc của ô Ngôi kể. Để trong chữ tự do thì
/// đổi ô xong prompt nói hai ý ngược nhau (truyện user báo 2026-11-21: luật "lời kể xưng tôi" cạnh bảng "我 → ta").
pub fn mentions_narrator_pronoun(text: &str) -> bool {
    NARRATOR_WORD.is_match(text) && SELF_PRONOUN.is_match(&GROUP_PRONOUN.replace_all(text, ""))
}

/// Đoạn có `我` ở lời kể hay không.
fn narrates_first_person(paragraph: &str) -> bool {
    let narration = QUOTE.replace_all(paragraph, "");
    THOUGHT.replace(&narration, "").contains('我')
}

pub fn guess_texts<'a>(chapters: impl IntoIterator<Item = &'a str>) -> PovGuess {
    let mut guess = PovGuess::default();
    for text in chapters {
        guess.chapters += 1;
        for paragraph in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
            guess.paragraphs += 1;
            guess.first_person += usize::from(narrates_first_person(paragraph));
        }
    }
    guess
}

/// Tối đa `SAMPLE_CHAPTERS` chương rải đều trong `raw/`; chương đọc lỗi thì bỏ qua.
pub fn guess_story(paths: &StoryPaths) -> Result<PovGuess> {
    let ids = list_raw_chapter_ids(paths)?;
    let step = (ids.len() / SAMPLE_CHAPTERS).max(1);
    let texts: Vec<String> = ids
        .iter()
        .step_by(step)
        .take(SAMPLE_CHAPTERS)
        .filter_map(|id| read_raw_chapter(paths, id).ok())
        .collect();
    Ok(guess_texts(texts.iter().map(String::as_str)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dem_我_o_loi_ke_bo_thoai_va_y_nghi_co_dan() {
        assert!(narrates_first_person("我走向了那边的诊室。"));
        assert!(narrates_first_person("我也只是朝她笑笑：“你在这等等，我去找我同学。”"));
        assert!(!narrates_first_person("“我去找我同学。”她说。"));
        assert!(!narrates_first_person("他心想：我完了。"));
        assert!(!narrates_first_person("「我不去」『我也不去』\"我更不去\""));
        assert!(narrates_first_person("沈涵不得不离我很近。"));
    }

    #[test]
    fn dong_style_noi_nguoi_ke_xung_gi() {
        for text in [
            "Lời kể ngôi thứ nhất xưng 'tôi', giữ giọng văn hiện đại gãy gọn, gần gũi, không dùng 'ta' trong lời dẫn truyện.",
            "Người kể tự xưng ta.",
            "Ngôi một, nữ chính xưng tôi.",
            "Kể chuyện ở ngôi thứ nhất, xưng \"tôi\"",
        ] {
            assert!(mentions_narrator_pronoun(text), "{text}");
        }
        for text in [
            "Lời kể ngôi ba dùng hắn / cô.",
            "Nữ chính xưng tôi với đồng nghiệp, gọi sếp là anh.",
            "Lời kể nhắc người ta, chúng ta, bọn tôi vẫn được.",
            "Giọng kể tưng tửng, nhiều câu cảm thán.",
            "Gọi cha mẹ là cha/mẹ, không bố.",
        ] {
            assert!(!mentions_narrator_pronoun(text), "{text}");
        }
    }

    #[test]
    fn doan_ngoi_ke_theo_nguong_va_goi_y_theo_boi_canh() {
        let first = "我走进门。\n“你来了？”他说。\n我点点头。\n".repeat(20);
        let third = "他走进门。\n“我来了。”他说。\n她点点头。\n".repeat(20);
        let guess = guess_texts([first.as_str()]);
        assert_eq!((guess.chapters, guess.paragraphs, guess.first_person), (1, 60, 40));
        assert_eq!(guess.suggested(GenreSetting::Modern), Some(StoryPov::FirstToi));
        assert_eq!(guess.suggested(GenreSetting::Ancient), Some(StoryPov::FirstTa));
        assert_eq!(guess.suggested(GenreSetting::Mixed), Some(StoryPov::FirstTa));
        let guess = guess_texts([third.as_str(), third.as_str()]);
        assert_eq!((guess.chapters, guess.paragraphs, guess.first_person), (2, 120, 0));
        assert_eq!(guess.suggested(GenreSetting::Modern), Some(StoryPov::Third));
        // Quá ít đoạn: không đoán.
        let few = guess_texts(["我走进门。\n我点点头。"]);
        assert_eq!(few.is_first_person(), None);
        assert_eq!(few.suggested(GenreSetting::Modern), None);
        assert_eq!(guess_texts(std::iter::empty()).ratio(), 0.0);
    }
}
