// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

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