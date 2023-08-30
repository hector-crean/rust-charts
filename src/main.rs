use charts::{
    create_crs_graph, errors,
    models::DosageEvent,
    sankey::{LayerOrderingMethod, SankeyLayers},
    sankey_graph::{convert_to_sankey, SankeyStyle},
};

fn labeller(dosage_event: DosageEvent) -> String {
    dosage_event.to_string()
}
fn main() -> errors::Result<()> {
    let graph = create_crs_graph()?;

    let sankey = convert_to_sankey(graph, &labeller);

    let style = SankeyStyle {
        number_format: Some(|x| format!("{x}")),
        ..SankeyStyle::default()
    };

    let svg = sankey.draw(512.0, 512.0, style);

    svg::save("./example.svg", &svg).unwrap();

    Ok(())
}
