use std::{fmt, fs::File, io::Read, path::Path};

use serde::{Deserialize, Serialize};

#[derive(
    Serialize, Deserialize, Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy, strum::Display,
)]
pub enum CytokineReleaseSyndromeGrade {
    G0,
    G1,
    G2,
}
#[derive(
    Serialize, Deserialize, Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy, strum::Display,
)]
pub enum Dose {
    D1,
    D2,
    D3,
    D4,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct NodeDatum {
    pub grade: CytokineReleaseSyndromeGrade,
    pub dose: Dose,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Node {
    pub id: String,
    // label: String,
    pub datum: NodeDatum,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EdgeDatum {
    pub weight: i64,
    pub subject_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Edge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub datum: EdgeDatum,
}

#[derive(Serialize, Deserialize)]
pub struct AltGraph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(thiserror::Error, Debug)]
pub enum DeserializeFromFileError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    // #[error(transparent)]

    // DeseraialiseError(#[from])
}

#[derive(Debug, Copy, Clone)]
pub struct DosageEvent {
    // id: &'a str,
    pub grade: CytokineReleaseSyndromeGrade,
    pub dose: Dose,
}

impl fmt::Display for DosageEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.grade, self.dose)
    }
}

pub fn deserialise_from_file<Json: Serialize + for<'a> Deserialize<'a>>(
    path: impl AsRef<Path>,
) -> Result<Json, DeserializeFromFileError> {
    let mut rdr = File::open(path)?;

    let json: Json = serde_json::from_reader(rdr).expect("JSON was not well-formatted");

    Ok(json)
}
