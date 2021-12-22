
//! Test that the auto-traits are implemented for certain public types.


fn is_normal<T: Sized + Send + Sync + Unpin> () {}

#[test]
fn chip_is_normal_type() {
    use libreda_db::chip::Chip;
    is_normal::<Chip>();
}

#[test]
fn rc_string_is_normal_type() {
    use libreda_db::rc_string::RcString;
    is_normal::<RcString>();
}