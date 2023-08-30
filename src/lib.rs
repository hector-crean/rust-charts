pub mod errors;
pub mod file_op;
pub mod models;
pub mod sankey;
pub mod sankey_graph;
pub mod settings;

use crate::{
    file_op::read_csv_file,
    models::{crs_dose::AeDoseCsvRecord, DosageEvent},
    models::{CytokineReleaseSyndromeGrade, Dose},
};
use petgraph::{stable_graph::NodeIndex, Directed, Graph};
use std::collections::HashMap;
use strum::IntoEnumIterator;

trait SortAndReturn {
    fn sort_and_return(self) -> Self;
}

impl<T: Ord> SortAndReturn for Vec<T> {
    fn sort_and_return(mut self) -> Self {
        self.sort();
        self
    }
}

pub fn create_crs_graph() -> errors::Result<Graph<DosageEvent, i32, Directed>> {
    let records: Vec<AeDoseCsvRecord> = read_csv_file("./dose.csv")?;

    let mut node_idxs = HashMap::<(Dose, CytokineReleaseSyndromeGrade), NodeIndex>::new();

    let mut graph = Graph::<DosageEvent, i32, Directed>::new();

    for dose in Dose::iter() {
        for grade in CytokineReleaseSyndromeGrade::iter() {
            let node_id = graph.add_node(DosageEvent { grade, dose });
            node_idxs.insert((dose, grade), node_id);
        }
    }

    let mut subjects = HashMap::<i32, Vec<AeDoseCsvRecord>>::new();

    for record in records.iter() {
        let key = record.subject_id;

        subjects
            .entry(key)
            .or_insert_with(Vec::new)
            .push(record.clone());
    }

    for (_, dose_events) in subjects.iter() {
        let sorted_dose_events = dose_events.clone().sort_and_return();

        for dose in sorted_dose_events.windows(2) {
            match &dose {
                &[source, target] => {
                    assert!(source.dose_number + 1 == target.dose_number);

                    let dose: Dose = source.dose_number.try_into()?;
                    let grade: CytokineReleaseSyndromeGrade =
                        source.cytokine_release_syndrome_grade_id.try_into()?;

                    let source_idx: Option<&NodeIndex> = node_idxs.get(&(dose, grade));

                    let dose: Dose = target.dose_number.try_into()?;
                    let grade: CytokineReleaseSyndromeGrade =
                        target.cytokine_release_syndrome_grade_id.try_into()?;

                    let target_idx: Option<&NodeIndex> = node_idxs.get(&(dose, grade));

                    match (source_idx, target_idx) {
                        (Some(&source_idx), Some(&target_idx)) => {
                            let _ = graph.add_edge(source_idx, target_idx, 1);
                        }
                        _ => {}
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    Ok(graph)
}
