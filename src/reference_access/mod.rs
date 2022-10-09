// Copyright (c) 2020-2021 Thomas Kramer.
// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! # Experimental
//! Wrappers around the [`crate::traits::HierarchyBase`], [`crate::traits::NetlistBase`], [`crate::traits::LayoutBase`] and [`crate::traits::L2NBase`] traits which
//! provide more object like access methods.
//!
//! # Examples
//!
//! ```
//! use libreda_db::prelude::*;
//!
//! // Create some netlist/layout.
//! let mut chip = Chip::new();
//! let top_id = chip.create_cell("TOP".into());
//! let sub_id = chip.create_cell("SUB".into());
//! let sub_inst1_id = chip.create_cell_instance(&top_id, &sub_id, Some("inst1".into()));
//!
//! // Create read-only object-like access.
//! let top = chip.cell_ref(&top_id);
//! // `top` can now be used like an object to navigate the cell hierarchy, layout and netlist.
//! for subcell in top.each_cell_instance() {
//!     println!("{} contains {:?} which is a {}", top.name(), subcell.name(), subcell.template().name());
//! }
//!
//! // Also the netlist can be traversed in a similar way.
//! for pin in top.each_pin() {
//!     println!("Pin {} of {} is connected to net {:?}.",
//!         pin.name(), top.name(), pin.net().and_then(|net| net.name())
//!     );
//! }
//! ```

mod hierarchy_reference_access;
mod l2n_reference_access;
mod layout_reference_access;
mod netlist_reference_access;

// Public exports.
pub use hierarchy_reference_access::*;
pub use l2n_reference_access::*;
pub use layout_reference_access::*;
pub use netlist_reference_access::*;

#[test]
fn test_chip_reference_access() {
    use crate::chip::Chip;
    use crate::prelude::*;

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
    assert_eq!(
        top_ref.each_net().count(),
        2,
        "LOW and HIGH nets should be there."
    );
    assert_eq!(top_ref.each_pin().count(), 1);
    assert_eq!(sub_inst1_ref.each_pin_instance().count(), 1);
}
