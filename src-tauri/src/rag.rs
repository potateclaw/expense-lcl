use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialTip {
    pub id: u32,
    pub category: String,
    pub tip: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TipsDatabase {
    pub tips: Vec<FinancialTip>,
}

pub struct RagRetriever {
    tips: Vec<FinancialTip>,
}

impl RagRetriever {
    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read tips file: {}", e))?;

        let db: TipsDatabase = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse tips JSON: {}", e))?;

        Ok(Self { tips: db.tips })
    }

    pub fn retrieve(&self, query: &str, top_k: usize) -> Vec<&FinancialTip> {
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        let mut scored: Vec<(&&FinancialTip, usize)> = self.tips.iter()
            .map(|tip| {
                let score = self.calculate_relevance(tip, &query_words);
                (tip, score)
            })
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1));

        scored.into_iter()
            .take(top_k)
            .map(|(tip, _)| tip)
            .collect()
    }

    fn calculate_relevance(&self, tip: &FinancialTip, query_words: &[&str]) -> usize {
        let mut score = 0usize;

        let category_lower = tip.category.to_lowercase();
        let tip_lower = tip.tip.to_lowercase();

        for word in query_words {
            if category_lower.contains(word) {
                score += 10;
            }

            if tip_lower.contains(word) {
                score += 5;
            }

            for tag in &tip.tags {
                if tag.to_lowercase().contains(word) {
                    score += 3;
                }
            }
        }

        let exact_category_match = query_words.iter()
            .any(|word| category_lower == *word);
        if exact_category_match {
            score += 15;
        }

        score
    }

    pub fn get_all_tips(&self) -> &[FinancialTip] {
        &self.tips
    }

    pub fn get_tip_count(&self) -> usize {
        self.tips.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relevance_scoring() {
        let tips = vec![
            FinancialTip {
                id: 1,
                category: "Food/Dining".to_string(),
                tip: "Cook at home to save money".to_string(),
                tags: vec!["cooking".to_string(), "savings".to_string()],
            },
            FinancialTip {
                id: 2,
                category: "Transportation".to_string(),
                tip: "Use public transit".to_string(),
                tags: vec!["commute".to_string(), "gas".to_string()],
            },
        ];

        let retriever = RagRetriever { tips };
        let results = retriever.retrieve("food cooking", 2);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, 1);
    }

    #[test]
    fn test_top_k_limit() {
        let tips = vec![
            FinancialTip {
                id: 1,
                category: "Food".to_string(),
                tip: "Test tip".to_string(),
                tags: vec![],
            },
            FinancialTip {
                id: 2,
                category: "Transport".to_string(),
                tip: "Test tip 2".to_string(),
                tags: vec![],
            },
            FinancialTip {
                id: 3,
                category: "Utilities".to_string(),
                tip: "Test tip 3".to_string(),
                tags: vec![],
            },
        ];

        let retriever = RagRetriever { tips };
        let results = retriever.retrieve("test", 2);

        assert_eq!(results.len(), 2);
    }
}
