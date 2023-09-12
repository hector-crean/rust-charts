use std::collections::HashMap;
use svgdom::{Attribute, AttributeId, AttributeValue, Document, ElementId, FilterSvg, PathCommand};

use charts::{
    create_crs_graph, errors,
    file_op::read_json_file,
    models::{crs_dose::AeDoseColorHashCsvRecord, DosageEvent},
    sankey::{LayerOrderingMethod, SankeyLayers},
    sankey_graph::{convert_to_sankey, SankeyStyle},
};

fn labeller(dosage_event: DosageEvent) -> String {
    dosage_event.to_string()
}
fn main() -> errors::Result<()> {
    let records: Vec<AeDoseColorHashCsvRecord> = read_json_file("./dose.csv")?;

    let mut color_hash_to_subject_id = HashMap::new();
    let mut subject_id_to_color_hash = HashMap::new();

    for AeDoseColorHashCsvRecord {
        subject_id,
        color_hash,
        ..
    } in &records
    {
        color_hash_to_subject_id.insert(color_hash, subject_id);
        subject_id_to_color_hash.insert(subject_id, color_hash);
    }

    let svg_str = include_str!("../sankey_diagram.svg");
    let doc = Document::from_str(&svg_str).unwrap();

    let mut count = 0;
    for (tag, mut node) in doc.root().descendants().svg() {
        match tag {
            ElementId::Polygon => {
                let attrs = node.attributes();
                if let Some(&AttributeValue::String(ref hash)) =
                    &attrs.get_value(AttributeId::Color)
                {
                    let subject_id = color_hash_to_subject_id.get(hash);

                    if let Some(&&subject_id) = subject_id {
                        node.set_attribute(Attribute::new(
                            AttributeId::FontFamily,
                            AttributeValue::Number(subject_id as f64),
                        ))
                    }
                }
            }
            _ => {}
        }
    }

    // svg::save("./example.svg", &svg).unwrap();

    Ok(())
}
