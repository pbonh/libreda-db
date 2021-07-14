
use libreda_db::prelude::*;

#[test]
fn test_shape_properties() {
    let mut layout = Chip::new();
    let layer = layout.find_or_create_layer(0, 0);
    let top_cell = layout.create_cell("TOP".into());
    let rect = Rect::new((0, 0), (10, 10));
    let rect_id = layout.insert_shape(&top_cell, &layer, rect.into());
    let key: RcString = "name".into();
    layout.set_shape_property(&rect_id, key.clone(), "Rectangle".into());

    assert_eq!(layout.get_shape_property(&rect_id, &key)
                   .and_then(|p| p.get_string()),
               Some("Rectangle".into()));
}