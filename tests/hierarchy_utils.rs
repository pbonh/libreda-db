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

//! Tests for `HierarchyUtils`

#![cfg(test)]

use libreda_db::chip::Chip;
use libreda_db::prelude::{HierarchyEdit, HierarchyBase};
use libreda_db::hierarchy_utils::*;

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
    assert_eq!(instances.len(), 1, "Expect to have one instance of C inside B.");
    assert!(chip.cell_by_name("C").is_some());
    for inst in instances {
        chip.prune_cell_instance(&inst);
    }
    assert!(chip.cell_by_name("C").is_none())
}