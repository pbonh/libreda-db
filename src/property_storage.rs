// Copyright (c) 2020-2021 Thomas Kramer.
// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Container structs for user defined properties.

use crate::rc_string::RcString;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::convert::TryInto;
use std::hash::Hash;
use std::sync::Arc;

// trait AnyValue: Any + Clone + std::fmt::Debug {}

/// Property value type.
/// Properties can hold different types that are encapsulated in this enum.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PropertyValue {
    /// Property is a string.
    String(RcString),
    /// Property is a byte string.
    Bytes(Vec<u8>),
    /// Property is a signed integer.
    SInt(i32),
    /// Property is an unsigned integer.
    UInt(u32),
    /// Property is a float.
    Float(f64),
    // /// Dynamically typed value.
    // Any(Box<dyn AnyValue>),
}

impl PropertyValue {
    /// Try to get a string value.
    pub fn get_string(&self) -> Option<RcString> {
        match self {
            PropertyValue::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    /// Try to get a `&str` value. Works for `String` property values.
    pub fn get_str(&self) -> Option<&str> {
        match self {
            PropertyValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Try to get a byte string value.
    pub fn get_bytes(&self) -> Option<&Vec<u8>> {
        match self {
            PropertyValue::Bytes(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get a float value.
    pub fn get_float(&self) -> Option<f64> {
        match self {
            PropertyValue::Float(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to get an i32 value.
    pub fn get_sint(&self) -> Option<i32> {
        match self {
            PropertyValue::SInt(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to get an i32 value.
    pub fn get_uint(&self) -> Option<u32> {
        match self {
            PropertyValue::UInt(v) => Some(*v),
            _ => None,
        }
    }

    // /// Try to get a dynamically typed value.
    // pub fn get_any(&self) -> Option<&Box<dyn AnyValue>> {
    //     match self {
    //         PropertyValue::Any(v) => Some(v),
    //         _ => None
    //     }
    // }
}

// pub enum PropertyKey {
//     String(String),
//
// }

impl From<String> for PropertyValue {
    fn from(v: String) -> Self {
        PropertyValue::String(v.into())
    }
}

impl From<Arc<String>> for PropertyValue {
    fn from(v: Arc<String>) -> Self {
        PropertyValue::String(v.into())
    }
}

impl From<&Arc<String>> for PropertyValue {
    fn from(v: &Arc<String>) -> Self {
        PropertyValue::String(v.into())
    }
}

impl From<&str> for PropertyValue {
    fn from(v: &str) -> Self {
        PropertyValue::String(v.into())
    }
}

impl From<Vec<u8>> for PropertyValue {
    fn from(v: Vec<u8>) -> Self {
        PropertyValue::Bytes(v)
    }
}

impl<'a> TryInto<&'a str> for &'a PropertyValue {
    type Error = ();

    fn try_into(self) -> Result<&'a str, Self::Error> {
        if let PropertyValue::String(s) = self {
            Ok(s.as_str())
        } else {
            Err(())
        }
    }
}

impl From<i32> for PropertyValue {
    fn from(v: i32) -> Self {
        PropertyValue::SInt(v)
    }
}

impl TryInto<i32> for &PropertyValue {
    type Error = ();

    fn try_into(self) -> Result<i32, Self::Error> {
        if let PropertyValue::SInt(v) = self {
            Ok(*v)
        } else {
            Err(())
        }
    }
}

impl From<u32> for PropertyValue {
    fn from(v: u32) -> Self {
        PropertyValue::UInt(v)
    }
}

impl From<f64> for PropertyValue {
    fn from(v: f64) -> Self {
        PropertyValue::Float(v)
    }
}

// impl From<Box<dyn Any>> for PropertyValue {
//     fn from(v: Box<dyn Any>) -> Self {
//         PropertyValue::Any(v)
//     }
// }

/// Look-up table for property values.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PropertyStore<K>
where
    K: Hash + Eq,
{
    content: HashMap<K, PropertyValue>,
}

impl<K: Hash + Eq> Default for PropertyStore<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Hash + Eq> PropertyStore<K> {
    /// Create an empty property store.
    pub fn new() -> Self {
        PropertyStore {
            content: HashMap::new(),
        }
    }

    /// Insert a property.
    /// Returns the old property value if there was already a property stored under this key.
    pub fn insert<V: Into<PropertyValue>>(&mut self, key: K, value: V) -> Option<PropertyValue> {
        self.content.insert(key, value.into())
    }

    /// Get a property value by the property key.
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&PropertyValue>
    where
        K: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.content.get(key)
    }

    /// Check if the `key` is contained in this property store.
    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.content.contains_key(key)
    }

    /// Get a string property value by key.
    /// If the property value is not a string `None` is returned.
    pub fn get_string<Q: ?Sized>(&self, key: &Q) -> Option<&RcString>
    where
        K: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.get(key).and_then(|v| {
            if let PropertyValue::String(s) = v {
                Some(s)
            } else {
                None
            }
        })
    }
}

/// A trait for associating user defined properties with a type.
pub trait WithProperties {
    /// Property key type.
    type Key: Hash + Eq;

    /// Call a function with maybe the property storage as argument.
    ///
    /// The property store might not always be initialized. For instance for
    /// objects without any defined properties, it will likely be `None`.
    fn with_properties<F, R>(&self, f: F) -> R
    where
        F: FnOnce(Option<&PropertyStore<Self::Key>>) -> R;

    /// Get mutable reference to the property storage.
    fn with_properties_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut PropertyStore<Self::Key>) -> R;

    /// Get a property value by the property key.
    fn property<Q: ?Sized>(&self, key: &Q) -> Option<PropertyValue>
    where
        Self::Key: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.with_properties(|p| p.and_then(|p| p.get(key).cloned()))
    }

    /// Get a string property value by key.
    /// If the property value is not a string `None` is returned.
    fn property_str<Q: ?Sized>(&self, key: &Q) -> Option<RcString>
    where
        Self::Key: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.with_properties(|p| p.and_then(|p| p.get_string(key).cloned()))
    }

    /// Insert a property.
    /// Returns the old property value if there was already a property stored under this key.
    fn set_property<V: Into<PropertyValue>>(
        &self,
        key: Self::Key,
        value: V,
    ) -> Option<PropertyValue> {
        self.with_properties_mut(|p| p.insert(key, value))
    }
}
