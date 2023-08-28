pub mod crs_dose;

use std::{fmt, fs::File, io::Write, path::Path};

use serde::{Deserialize, Serialize};

#[derive(
    Hash,
    Serialize,
    Deserialize,
    Debug,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    Clone,
    Copy,
    strum::Display,
    strum::EnumIter,
)]
pub enum CytokineReleaseSyndromeGrade {
    G0,
    G1,
    G2,
}

#[derive(thiserror::Error, Debug)]
pub enum EnumIntConversionError {
    #[error("The integer`{0}` could not be converted into the enum")]
    FromIntError(i32),
}
impl TryFrom<i32> for CytokineReleaseSyndromeGrade {
    type Error = EnumIntConversionError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        use CytokineReleaseSyndromeGrade::*;
        match value {
            0 => Ok(G0),
            1 => Ok(G1),
            2 => Ok(G2),
            int @ _ => Err(EnumIntConversionError::FromIntError(int)),
        }
    }
}

#[derive(
    Hash,
    Serialize,
    Deserialize,
    Debug,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    Clone,
    Copy,
    strum::Display,
    strum::EnumIter,
)]
pub enum Dose {
    D1,
    D2,
    D3,
    D4,
}

impl TryFrom<i32> for Dose {
    type Error = EnumIntConversionError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        use Dose::*;
        match value {
            1 => Ok(D1),
            2 => Ok(D2),
            3 => Ok(D3),
            4 => Ok(D4),
            int @ _ => Err(EnumIntConversionError::FromIntError(int)),
        }
    }
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
    let rdr = File::open(path)?;

    let json: Json = serde_json::from_reader(rdr).expect("JSON was not well-formatted");

    Ok(json)
}

pub fn save_to_file(path: impl AsRef<Path>, content: &str) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}
