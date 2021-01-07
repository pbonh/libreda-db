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

//! `RcString` is a simple data structure for the representation of strings.
//! In contrast to `String`, `RcString` can be efficiently cloned. It is intended
//! to be used in cases where objects are indexed by a human readable name.
//!
//! # Example
//!
//! ```
//! use libreda_db::rc_string::RcString;
//!
//! let a: String = "A".to_string();
//!
//! let a1_rc = RcString::from(a);
//! let a2_rc = RcString::from("A");
//!
//! // No string data is copied here.
//! let a3_rc = a1_rc.clone();
//!
//! assert_eq!(a1_rc, a2_rc);
//! assert_eq!(a1_rc, a3_rc);
//!
//! ```

use std::rc::Rc;
use iron_shapes::point::Deref;
use std::borrow::Borrow;
use std::hash::{Hash, Hasher};

/// Resource counted string, used for names.
/// `RcString`s can be efficiently cloned.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RcString {
    string: Rc<String>
}

impl std::fmt::Display for RcString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.string.as_str(), f)
    }
}

impl RcString {
    /// Create a new resource counted string.
    pub fn new(string: String) -> Self {
        RcString { string: Rc::new(string) }
    }
}

impl Hash for RcString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.string.hash(state)
    }
}

impl Deref for RcString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        self.string.deref()
    }
}

impl Borrow<str> for RcString {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<String> for RcString {
    fn borrow(&self) -> &String {
        self.string.deref()
    }
}

impl From<String> for RcString {
    fn from(string: String) -> Self {
        Self::new(string)
    }
}

impl From<Rc<String>> for RcString {
    fn from(string: Rc<String>) -> Self {
        Self { string }
    }
}

impl From<&str> for RcString {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

impl Into<String> for RcString {
    fn into(self) -> String {
        self.string.to_string()
    }
}
