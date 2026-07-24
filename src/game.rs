use std::collections::{HashMap, HashSet};

const DICTIONARY: &str = include_str!("../assets/enable1.txt");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Puzzle {
    pub letters: Vec<char>,
    pub center: char,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SubmitError {
    TooShort,
    MissingCenter,
    InvalidLetters,
    NotInDictionary,
    AlreadyFound,
}

impl SubmitError {
    pub fn message(&self) -> &'static str {
        match self {
            SubmitError::TooShort => "Too short",
            SubmitError::MissingCenter => "Missing center letter",
            SubmitError::InvalidLetters => "Invalid letters",
            SubmitError::NotInDictionary => "Not in word list",
            SubmitError::AlreadyFound => "Already found",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WordEntry {
    pub word: String,
    pub points: u32,
    pub pangram: bool,
}

#[derive(Clone, Debug)]
pub struct PuzzleStats {
    pub words: Vec<WordEntry>,
    pub max_score: u32,
    pub pangram_count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rank {
    pub name: &'static str,
    pub threshold_pct: u32,
}

pub const RANKS: [Rank; 10] = [
    Rank { name: "Beginner", threshold_pct: 0 },
    Rank { name: "Good Start", threshold_pct: 2 },
    Rank { name: "Moving Up", threshold_pct: 5 },
    Rank { name: "Good", threshold_pct: 8 },
    Rank { name: "Solid", threshold_pct: 15 },
    Rank { name: "Nice", threshold_pct: 25 },
    Rank { name: "Great", threshold_pct: 40 },
    Rank { name: "Amazing", threshold_pct: 50 },
    Rank { name: "Genius", threshold_pct: 70 },
    Rank { name: "Queen Bee", threshold_pct: 100 },
];

pub fn parse_puzzle(raw_letters: &str, center: char) -> Option<Puzzle> {
    let mut letters: Vec<char> = raw_letters
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| c.to_ascii_lowercase())
        .collect();

    if letters.len() != 7 {
        return None;
    }

    letters.sort_unstable();
    letters.dedup();

    if letters.len() != 7 {
        return None;
    }

    let center = center.to_ascii_lowercase();
    if !letters.contains(&center) {
        return None;
    }

    Some(Puzzle { letters, center })
}

pub fn outer_letters(puzzle: &Puzzle) -> Vec<char> {
    puzzle
        .letters
        .iter()
        .copied()
        .filter(|letter| *letter != puzzle.center)
        .collect()
}

pub fn is_pangram(word: &str, puzzle: &Puzzle) -> bool {
    let used: HashSet<char> = word.chars().collect();
    puzzle.letters.iter().all(|letter| used.contains(letter))
}

pub fn score_word(word: &str, pangram: bool) -> u32 {
    let len = word.chars().count() as u32;
    let base = if len == 4 { 1 } else { len };
    if pangram {
        base + 7
    } else {
        base
    }
}

pub fn validate_submission(
    raw: &str,
    puzzle: &Puzzle,
    found: &HashSet<String>,
    dictionary: &HashSet<&str>,
) -> Result<WordEntry, SubmitError> {
    let word: String = raw
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| c.to_ascii_lowercase())
        .collect();

    if word.chars().count() < 4 {
        return Err(SubmitError::TooShort);
    }

    if !word.contains(puzzle.center) {
        return Err(SubmitError::MissingCenter);
    }

    let allowed: HashSet<char> = puzzle.letters.iter().copied().collect();
    if !word.chars().all(|c| allowed.contains(&c)) {
        return Err(SubmitError::InvalidLetters);
    }

    if !dictionary.contains(word.as_str()) {
        return Err(SubmitError::NotInDictionary);
    }

    if found.contains(&word) {
        return Err(SubmitError::AlreadyFound);
    }

    let pangram = is_pangram(&word, puzzle);
    let points = score_word(&word, pangram);

    Ok(WordEntry {
        word,
        points,
        pangram,
    })
}

pub fn analyze_puzzle(puzzle: &Puzzle) -> PuzzleStats {
    let dictionary = load_dictionary();
    let allowed: HashSet<char> = puzzle.letters.iter().copied().collect();
    let mut words = Vec::new();

    for entry in dictionary {
        if entry.chars().count() < 4 {
            continue;
        }
        if !entry.contains(puzzle.center) {
            continue;
        }
        if !entry.chars().all(|c| allowed.contains(&c)) {
            continue;
        }

        let pangram = is_pangram(entry, puzzle);
        let points = score_word(entry, pangram);
        words.push(WordEntry {
            word: entry.to_string(),
            points,
            pangram,
        });
    }

    words.sort_by(|a, b| a.word.cmp(&b.word));

    let max_score = words.iter().map(|w| w.points).sum();
    let pangram_count = words.iter().filter(|w| w.pangram).count() as u32;

    PuzzleStats {
        words,
        max_score,
        pangram_count,
    }
}

pub fn current_rank(score: u32, max_score: u32) -> &'static str {
    if max_score == 0 {
        return "Beginner";
    }

    let pct = score.saturating_mul(100) / max_score;
    let mut current = RANKS[0].name;

    for rank in RANKS.iter() {
        if pct >= rank.threshold_pct {
            current = rank.name;
        }
    }

    current
}

pub fn next_rank(score: u32, max_score: u32) -> Option<(&'static str, u32)> {
    if max_score == 0 {
        return None;
    }

    let pct = score.saturating_mul(100) / max_score;

    for rank in RANKS.iter() {
        if pct < rank.threshold_pct {
            let needed_score = rank.threshold_pct.saturating_mul(max_score).div_ceil(100);
            return Some((rank.name, needed_score.saturating_sub(score)));
        }
    }

    None
}

pub fn rank_progress(score: u32, max_score: u32) -> f64 {
    if max_score == 0 {
        return 0.0;
    }
    (score as f64 / max_score as f64).clamp(0.0, 1.0)
}

pub fn rank_markers(max_score: u32) -> Vec<(&'static str, u32)> {
    RANKS
        .iter()
        .map(|rank| {
            let points = rank.threshold_pct.saturating_mul(max_score).div_ceil(100);
            (rank.name, points)
        })
        .collect()
}

pub fn shuffle_letters(puzzle: &Puzzle, salt: u64) -> Vec<char> {
    let mut outer = outer_letters(puzzle);
    let n = outer.len();
    if n <= 1 {
        return outer;
    }

    let mut state = salt.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(1);
    for i in (1..n).rev() {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let j = (state as usize) % (i + 1);
        outer.swap(i, j);
    }

    outer
}

pub fn load_dictionary() -> HashSet<&'static str> {
    static DICT: std::sync::OnceLock<HashSet<&'static str>> = std::sync::OnceLock::new();
    DICT.get_or_init(|| {
        DICTIONARY
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect()
    })
    .clone()
}

pub fn group_words_by_length(entries: &[WordEntry]) -> HashMap<usize, Vec<WordEntry>> {
    let mut groups: HashMap<usize, Vec<WordEntry>> = HashMap::new();
    for entry in entries {
        groups
            .entry(entry.word.chars().count())
            .or_default()
            .push(entry.clone());
    }
    for words in groups.values_mut() {
        words.sort_by(|a, b| a.word.cmp(&b.word));
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_puzzle() -> Puzzle {
        parse_puzzle("AIRBNST", 'n').unwrap()
    }

    #[test]
    fn parses_valid_puzzle() {
        let puzzle = sample_puzzle();
        assert_eq!(puzzle.letters.len(), 7);
        assert_eq!(puzzle.center, 'n');
    }

    #[test]
    fn scores_like_nyt() {
        assert_eq!(score_word("rain", false), 1);
        assert_eq!(score_word("train", false), 5);
        assert_eq!(score_word("brain", true), 12);
    }

    #[test]
    fn rejects_short_words() {
        let puzzle = sample_puzzle();
        let dict = load_dictionary();
        let found = HashSet::new();
        assert_eq!(
            validate_submission("ant", &puzzle, &found, &dict),
            Err(SubmitError::TooShort)
        );
    }
}
