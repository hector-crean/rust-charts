use csv::{ReaderBuilder, WriterBuilder};
use serde::{self, Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use std::io::Read;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JsonReadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    Parsing(#[from] serde_json::Error),
}

pub fn read_json_file<P: AsRef<Path>, T>(file_path: P) -> Result<Vec<T>, JsonReadError>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let records: Vec<T> = serde_json::from_str(&contents)?;

    Ok(records)
}

pub fn read_csv_file<P: AsRef<Path>, T>(file_path: P) -> Result<Vec<T>, csv::Error>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    let file = File::open(file_path)?;

    let mut csv_reader = ReaderBuilder::new().flexible(false).from_reader(file);

    let mut records = Vec::<T>::new();

    for record in csv_reader.deserialize::<T>() {
        let record = record?;
        records.push(record);
    }

    Ok(records)
}

pub fn write_csv_file<P: AsRef<Path>, T>(file_path: P, records: Vec<T>) -> Result<(), csv::Error>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    let file = File::open(file_path)?;

    let mut csv_writer = WriterBuilder::new().flexible(false).from_writer(file);

    for record in &records {
        csv_writer.serialize(record)?;
    }

    csv_writer.flush()?;

    Ok(())
}

pub fn write_to_file<P, T>(file_path: P, value: T) -> std::io::Result<()>
where
    P: AsRef<Path>,
    T: Serialize,
{
    let file = File::create(file_path)?;

    let mut writer = BufWriter::new(file);

    serde_json::to_writer(&mut writer, &value)?;

    writer.flush()?;

    Ok(())
}
