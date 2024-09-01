use std::cmp;
use unicode_segmentation::UnicodeSegmentation;

const SCORE_MATCH: f64 = 16.0;
const SCORE_GAP_EXTENSION: f64 = -1.0;

const BONUS_CONSECUTIVE: f64 = 4.0;
const BONUS_SLASH: f64 = 3.0;
const BONUS_WORD: f64 = 20.0;
const BONUS_CAMEL: f64 = 2.0;
const BONUS_DOT: f64 = 1.0;
const BONUS_FIRST_CHAR_MATCH: f64 = 50.0;
const BONUS_CASE_MATCH: f64 = 0.0;
const BONUS_ACRONYM: f64 = 80.0;

pub fn match_and_score(haystack: &str, needle: &str) -> Option<f64> {
    if needle.is_empty() {
        return Some(0.0);
    }

    let haystack_lower = haystack.to_lowercase();
    let haystack_chars: Vec<char> = haystack_lower.chars().collect();
    let needle_chars: Vec<char> = needle.chars().collect();

    if is_match(&haystack_chars, &needle_chars) {
        Some(compute_score(haystack, &haystack_chars, &needle_chars, needle))
    } else {
        None
    }
}

fn is_match(haystack: &[char], needle: &[char]) -> bool {
    let mut hi = 0;
    for &n in needle {
        while hi < haystack.len() && haystack[hi] != n {
            hi += 1;
        }
        if hi == haystack.len() {
            return false;
        }
        hi += 1;
    }
    true
}

fn compute_score(haystack: &str, haystack_chars: &[char], needle_chars: &[char], needle: &str) -> f64 {
    let n = needle_chars.len();
    let m = haystack_chars.len();

    let mut score = vec![vec![0.0; m]; n];
    let mut d = vec![vec![0.0; m]; n];

    // Check for acronym match
    let acronym_bonus = if is_acronym_match(haystack, needle_chars) {
        BONUS_ACRONYM * (n as f64)
    } else {
        0.0
    };

    // Check for case-sensitive whole word match
    let case_match_bonus = if haystack.contains(needle) {
        BONUS_CASE_MATCH
    } else {
        0.0
    };

    // Initialize first row
    for j in 0..m {
        if needle_chars[0].to_lowercase().next() == Some(haystack_chars[j]) {
            let mut match_score = SCORE_MATCH + bonus_for(haystack, j);
            if j == 0 {
                match_score += BONUS_FIRST_CHAR_MATCH;
            }
            score[0][j] = match_score;
            d[0][j] = score[0][j];
        } else {
            score[0][j] = 0.0;
            d[0][j] = 0.0;
        }
    }

    // Fill in the rest of the matrix
    for i in 1..n {
        for j in i..m {
            if needle_chars[i].to_lowercase().next() == Some(haystack_chars[j]) {
                let mut match_score = SCORE_MATCH + bonus_for(haystack, j);
                if i == j {
                    match_score += BONUS_CONSECUTIVE * (n as f64); // Bonus for exact match
                }
                d[i][j] = match_score + if i > 0 && j > 0 { d[i-1][j-1] } else { 0.0 };
                score[i][j] = cmp::max_by(
                    d[i][j],
                    score[i-1][j] + SCORE_GAP_EXTENSION,
                    |a, b| a.partial_cmp(b).unwrap()
                );
            } else {
                d[i][j] = 0.0;
                score[i][j] = score[i][j-1] + SCORE_GAP_EXTENSION;
            }
        }
    }

    score[n-1][m-1] + acronym_bonus + case_match_bonus
}

fn is_acronym_match(haystack: &str, needle_chars: &[char]) -> bool {
    let words: Vec<&str> = haystack.split_whitespace().collect();
    let mut needle_iter = needle_chars.iter();

    for word in words {
        if let Some(first_char) = word.chars().next() {
            if let Some(&needle_char) = needle_iter.next() {
                if first_char.to_lowercase().next() != Some(needle_char.to_lowercase().next().unwrap()) {
                    return false;
                }
            } else {
                // We've matched all needle characters
                return true;
            }
        }
    }

    // Check if we've used all needle characters
    needle_iter.next().is_none()
}

fn bonus_for(haystack: &str, index: usize) -> f64 {
    let mut bonus = 0.0;
    let graphemes: Vec<&str> = haystack.graphemes(true).collect();

    if index == 0 || is_boundary(graphemes[index - 1]) {
        bonus += BONUS_WORD;
    }

    if index > 0 {
        match graphemes[index - 1] {
            "/" => bonus += BONUS_SLASH,
            "." => bonus += BONUS_DOT,
            _ => {}
        }
    }

    if index > 0 && graphemes[index].chars().next().unwrap().is_uppercase() &&
       graphemes[index - 1].chars().next().unwrap().is_lowercase() {
        bonus += BONUS_CAMEL;
    }

    if index > 0 && graphemes[index] == graphemes[index - 1] {
        bonus += BONUS_CONSECUTIVE;
    }

    bonus
}

fn is_boundary(s: &str) -> bool {
    matches!(s, " " | "_" | "-")
}