// Copyright (c) 2020-2022 Thomas Kramer.
// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! ID of an arc (net segment). The arc is defined by two terminals (pin or pin instance).

use super::prelude::*;
use std::hash::Hash;

/// An arc represents the direct path from one pin to another.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ArcId<N: NetlistBase + ?Sized> {
    start: TerminalId<N>,
    end: TerminalId<N>,
}
