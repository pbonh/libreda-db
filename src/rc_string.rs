// Copyright (c) 2020-2021 Thomas Kramer.
// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

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

use iron_shapes::point::Deref;
use std::borrow::Borrow;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Resource counted string, used for names.
/// `RcString`s can be efficiently cloned.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RcString {
    string: Arc<String>,
}

impl std::fmt::Display for RcString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.string.as_str(), f)
    }
}

impl RcString {
    /// Create a new resource counted string.
    pub fn new(string: String) -> Self {
        RcString {
            string: Arc::new(string),
        }
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

impl From<Arc<String>> for RcString {
    fn from(string: Arc<String>) -> Self {
        Self { string }
    }
}

impl From<&Arc<String>> for RcString {
    fn from(string: &Arc<String>) -> Self {
        Self {
            string: string.clone(),
        }
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
