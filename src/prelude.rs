// Copyright (c) 2020-2021 Thomas Kramer.
// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The `prelude` helps to import most commonly used modules.

pub use iron_shapes::prelude::*;
pub use crate::hierarchy::prelude::*;
pub use crate::layout::prelude::*;
pub use crate::netlist::prelude::*;
pub use crate::l2n::*;
pub use crate::chip::Chip;
pub use crate::traits::*;
pub use crate::rc_string::RcString;
pub use crate::property_storage::PropertyValue;
pub use crate::flat_view::FlatView;
pub use crate::reference_access;
pub use crate::reference_access::*;
pub use crate::netlist::util::*;
pub use crate::technology;
pub use crate::technology::prelude::*;

/// Re-export of most traits.
/// This can be useful if only traits should be used but not the rest.
pub mod traits {
    pub use crate::traits::*;
    pub use crate::hierarchy::traits::*;
    pub use crate::hierarchy::util::*;
    pub use crate::layout::traits::*;
    pub use crate::layout::util::*;
    pub use crate::netlist::traits::*;
    pub use crate::netlist::util::*;
    pub use crate::l2n::*;
    pub use crate::reference_access::*;
    pub use iron_shapes::traits::*;
}
