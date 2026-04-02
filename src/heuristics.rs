//! Heuristics for tree traversal scoring

use serde::{Deserialize, Serialize};

/// Scoring heuristics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heuristics {
    /// Weight for title matches
    pub title_weight: f32,
    /// Weight for summary matches
    pub summary_weight: f32,
    /// Weight for body matches
    pub body_weight: f32,
    /// Weight for code matches
    pub code_weight: f32,
    /// Boost for rare terms (IDF)
    pub idf_boost: f32,
    /// Demotion factor for generic sections
    pub generic_demote: f32,
    /// Boost for parent nodes
    pub parent_boost: f32,
    /// Boost for sibling nodes
    pub sibling_boost: f32,
}

impl Default for Heuristics {
    fn default() -> Self {
        Self {
            title_weight: 10.0,
            summary_weight: 5.0,
            body_weight: 1.0,
            code_weight: 2.0,
            idf_boost: 1.5,
            generic_demote: 0.5,
            parent_boost: 1.2,
            sibling_boost: 1.1,
        }
    }
}

impl Heuristics {
    /// Check if a title is a generic section name
    pub fn is_generic_title(title: &str) -> bool {
        const GENERIC_TITLES: &[&str] = &[
            "introduction",
            "overview",
            "summary",
            "conclusion",
            "background",
            "related work",
            "references",
            "appendix",
            "前言",
            "概述",
            "简介",
            "总结",
            "结论",
            "附录",
            "参考文献",
        ];

        let lower = title.to_lowercase();
        GENERIC_TITLES.iter().any(|&t| lower.contains(t))
    }

    /// Calculate score for a title match
    pub fn score_title(&self, title: &str, query: &str) -> f32 {
        let mut score = self.title_weight;

        // Demote generic titles
        if Self::is_generic_title(title) {
            score *= self.generic_demote;
        }

        // Exact match bonus
        if title.to_lowercase() == query.to_lowercase() {
            score *= 2.0;
        }

        score
    }

    /// Calculate score for a text match
    pub fn score_text(&self, text: &str, query: &str, is_code: bool) -> f32 {
        let base_weight = if is_code {
            self.code_weight
        } else {
            self.body_weight
        };

        // Count occurrences
        let lower_text = text.to_lowercase();
        let lower_query = query.to_lowercase();
        let count = lower_text.matches(&lower_query).count() as f32;

        base_weight * count
    }

    /// Calculate path score based on node positions
    pub fn score_path(&self, path_length: usize, matched_position: usize) -> f32 {
        // Closer to root = higher score
        let depth_factor = 1.0 / (1.0 + matched_position as f32);
        let path_factor = 1.0 / (1.0 + path_length as f32).sqrt();

        depth_factor * path_factor
    }
}

/// IDF calculator for term rarity
pub struct IdfCalculator {
    total_docs: usize,
    term_doc_counts: std::collections::HashMap<String, usize>,
}

impl IdfCalculator {
    /// Create a new IDF calculator
    pub fn new(total_docs: usize) -> Self {
        Self {
            total_docs,
            term_doc_counts: std::collections::HashMap::new(),
        }
    }

    /// Add a term occurrence
    pub fn add_term(&mut self, term: &str) {
        *self.term_doc_counts.entry(term.to_lowercase()).or_insert(0) += 1;
    }

    /// Calculate IDF for a term
    pub fn idf(&self, term: &str) -> f32 {
        let lower = term.to_lowercase();
        let doc_count = self.term_doc_counts.get(&lower).copied().unwrap_or(1);

        if doc_count == 0 || self.total_docs == 0 {
            return 0.0;
        }

        // Standard IDF formula: log(N / df)
        ((self.total_docs as f32) / (doc_count as f32)).ln()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_heuristics() {
        let h = Heuristics::default();
        assert_eq!(h.title_weight, 10.0);
        assert_eq!(h.body_weight, 1.0);
    }

    #[test]
    fn test_generic_title() {
        assert!(Heuristics::is_generic_title("Introduction"));
        assert!(Heuristics::is_generic_title("前言"));
        assert!(!Heuristics::is_generic_title("Implementation"));
    }

    #[test]
    fn test_idf_calculator() {
        let mut calc = IdfCalculator::new(100);
        for _ in 0..10 {
            calc.add_term("rust");
        }
        let idf = calc.idf("rust");
        assert!(idf > 0.0);
    }
}