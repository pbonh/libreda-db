// Copyright (c) 2020-2021 Thomas Kramer.
// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests for `HierarchyUtils`

#![cfg(test)]

use libreda_db::chip::Chip;
use libreda_db::hierarchy::prelude::*;
use libreda_db::prelude::{HierarchyBase, HierarchyEdit};

/// Create a chip with three cells A, B, C and D.
///
/// A uses B
/// B uses C
fn create_test_chip_with_unused_subcell() -> Chip {
    let mut chip = Chip::new();
    let a = chip.create_cell("A".into());
    let b = chip.create_cell("B".into());
    let c = chip.create_cell("C".into());

    chip.create_cell_instance(&a, &b, None);
    chip.create_cell_instance(&b, &c, None);
    chip
}

/// Create a chip with three cells A, B, C and D.
///
/// A uses B, D
/// B uses C, D
fn create_test_chip_with_used_subcell() -> Chip {
    let mut chip = Chip::new();
    let a = chip.create_cell("A".into());
    let b = chip.create_cell("B".into());
    let c = chip.create_cell("C".into());
    let d = chip.create_cell("D".into());

    chip.create_cell_instance(&a, &b, None);
    chip.create_cell_instance(&a, &d, None);
    chip.create_cell_instance(&b, &c, None);
    chip.create_cell_instance(&b, &d, None);
    chip
}

#[test]
fn test_prune_cell_with_unused_subcell() {
    let mut chip = create_test_chip_with_unused_subcell();
    let b = chip.cell_by_name("B").unwrap();
    chip.prune_cell(&b);
    assert_eq!(chip.num_cells(), 1); // A needs to be present.
}

#[test]
fn test_prune_cell_with_used_subcell() {
    let mut chip = create_test_chip_with_used_subcell();
    let b = chip.cell_by_name("B").unwrap();
    chip.prune_cell(&b);
    assert_eq!(chip.num_cells(), 2); // A and D need to be present.
}

#[test]
fn test_prune_cell_instance() {
    let mut chip = create_test_chip_with_unused_subcell();
    let b = chip.cell_by_name("B").unwrap();
    let instances = chip.each_cell_instance_vec(&b);
    assert_eq!(
        instances.len(),
        1,
        "Expect to have one instance of C inside B."
    );
    assert!(chip.cell_by_name("C").is_some());
    for inst in instances {
        chip.prune_cell_instance(&inst);
    }
    assert!(chip.cell_by_name("C").is_none())
}
