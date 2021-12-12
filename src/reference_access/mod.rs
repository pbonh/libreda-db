/*
 * Copyright (c) 2020-2021 Thomas Kramer.
 *
 * This file is part of LibrEDA
 * (see https://codeberg.org/libreda).
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

//! # Experimental
//! Wrappers around the `HierarchyBase` and `NetlistBase` traits which
//! provide more object like access methods.
//!
//! # Examples
//!
//! ```
//! use libreda_db::prelude::*;
//! ```

mod hierarchy_reference_access;
mod netlist_reference_access;
mod layout_reference_access;
mod l2n_reference_access;

pub use hierarchy_reference_access::*;
pub use netlist_reference_access::*;
pub use layout_reference_access::*;
pub use l2n_reference_access::*;

#[test]
fn test_chip_reference_access() {
    use crate::prelude::*;
    use crate::chip::Chip;

    let mut chip = Chip::new();
    let top = chip.create_cell("TOP".into());
    chip.create_pin(&top, "A".into(), Direction::Input);
    let sub = chip.create_cell("SUB".into());
    chip.create_pin(&sub, "B".into(), Direction::Input);
    let sub_inst1 = chip.create_cell_instance(&top, &sub, Some("inst1".into()));

    let top_ref = chip.cell_ref(&top);
    assert_eq!(&top_ref.id(), &top);

    let sub_inst1_ref = chip.cell_instance_ref(&sub_inst1);
    assert_eq!(&sub_inst1_ref.id(), &sub_inst1);
    assert_eq!(sub_inst1_ref.parent().id(), top_ref.id());
    assert_eq!(&sub_inst1_ref.template().id(), &sub);

    // Access nets and pins.
    assert_eq!(top_ref.each_net().count(), 2, "LOW and HIGH nets should be there.");
    assert_eq!(top_ref.each_pin().count(), 1);
    assert_eq!(sub_inst1_ref.each_pin_instance().count(), 1);
}
