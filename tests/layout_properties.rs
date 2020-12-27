
use libreda_db::layout::prelude::*;
use libreda_db::property_storage::WithProperties;

#[test]
fn test_shape_properties() {
    let mut layout = Layout::new();
    let layer = layout.find_or_create_layer(0, 0);
    let top_cell = layout.create_and_get_cell(Some("TOP"));
    let shapes = top_cell.shapes_get_or_create(layer);
    let rect = shapes.insert(Rect::new((0, 0), (10, 10)));
    rect.set_property("name".to_string(), "Rectangle".to_string());
}