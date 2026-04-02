//! Tokenization for CJK and English text

use jieba_rs::Jieba;
use std::sync::OnceLock;

static JIEBA: OnceLock<Jieba> = OnceLock::new();

/// Get the global Jieba instance
fn get_jieba() -> &'static Jieba {
    JIEBA.get_or_init(Jieba::new)
}

/// Tokenize text into words
pub fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();

    for word in text.split_whitespace() {
        // Check if word contains CJK characters
        if contains_cjk(word) {
            // Use jieba for CJK segmentation
            let jieba = get_jieba();
            let words = jieba.cut(word, true);
            for w in words {
                let trimmed = w.trim();
                if !trimmed.is_empty() && !is_stop_word(trimmed) {
                    tokens.push(trimmed.to_lowercase());
                }
            }
        } else {
            // Simple tokenization for non-CJK
            let word = word.to_lowercase();
            if !is_stop_word(&word) {
                tokens.push(word);
            }
        }
    }

    tokens
}

/// Check if text contains CJK characters
fn contains_cjk(s: &str) -> bool {
    s.chars().any(|c| {
        // CJK Unified Ideographs
        (c >= '\u{4E00}' && c <= '\u{9FFF}') ||
        // CJK Unified Ideographs Extension A
        (c >= '\u{3400}' && c <= '\u{4DBF}') ||
        // CJK Unified Ideographs Extension B-F
        (c >= '\u{20000}' && c <= '\u{2CEAF}') ||
        // Hiragana
        (c >= '\u{3040}' && c <= '\u{309F}') ||
        // Katakana
        (c >= '\u{30A0}' && c <= '\u{30FF}') ||
        // Hangul
        (c >= '\u{AC00}' && c <= '\u{D7AF}')
    })
}

/// Check if a word is a stop word
fn is_stop_word(word: &str) -> bool {
    // Common English stop words
    const STOP_WORDS: &[&str] = &[
        "a", "an", "the", "and", "or", "but", "in", "on", "at", "to", "for",
        "of", "with", "by", "from", "as", "is", "was", "are", "were", "been",
        "be", "have", "has", "had", "do", "does", "did", "will", "would",
        "could", "should", "may", "might", "must", "shall", "can", "need",
        "this", "that", "these", "those", "it", "its", "they", "them", "their",
        "he", "him", "his", "she", "her", "we", "us", "our", "you", "your",
        "i", "me", "my", "what", "which", "who", "whom", "when", "where",
        "why", "how", "all", "each", "every", "both", "few", "more", "most",
        "other", "some", "such", "no", "nor", "not", "only", "own", "same",
        "so", "than", "too", "very", "just", "also", "now", "here", "there",
    ];

    STOP_WORDS.contains(&word)
}

/// Tokenize for indexing (more aggressive)
pub fn tokenize_for_index(text: &str) -> Vec<String> {
    let mut tokens = tokenize(text);

    // Add n-grams for better matching
    let words: Vec<&str> = text.split_whitespace().collect();
    for i in 0..words.len().saturating_sub(1) {
        let bigram = format!("{} {}", words[i], words[i + 1]);
        tokens.push(bigram.to_lowercase());
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_english() {
        let tokens = tokenize("Hello World from Rust");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"rust".to_string()));
    }

    #[test]
    fn test_tokenize_chinese() {
        let tokens = tokenize("我爱编程");
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_contains_cjk() {
        assert!(contains_cjk("中文"));
        assert!(contains_cjk("日本語"));
        assert!(contains_cjk("한국어"));
        assert!(!contains_cjk("English"));
    }
}