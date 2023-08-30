
use serde::{self, Deserialize, Serialize};
use std::{cmp::Ordering};

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq, PartialOrd)]
pub struct AeDoseCsvRecord {
    #[serde(alias = "NSID")]
    pub subject_id: i32,
    #[serde(alias = "AEDOSE")]
    pub dose_number: i32,
    #[serde(alias = "DV")]
    pub cytokine_release_syndrome_grade_id: i32,
    #[serde(alias = "DATE")]
    pub date: String,
    #[serde(alias = "TIME")]
    #[serde(deserialize_with = "csv::invalid_option")]
    pub time: Option<String>,
}

impl Ord for AeDoseCsvRecord {
    fn cmp(&self, other: &Self) -> Ordering {
        self.dose_number.cmp(&other.dose_number)
    }
}
