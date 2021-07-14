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

//! Container structs for user defined properties.

// use std::any::Any;
use std::collections::HashMap;
use std::hash::Hash;
use std::borrow::Borrow;
use std::convert::TryInto;
use std::rc::Rc;

/// Property value type.
/// Properties can hold different types that are encapsulated in this enum.
#[derive(Debug, Clone)]
pub enum PropertyValue {
    /// Property is a string.
    String(Rc<String>),
    /// Property is a byte string.
    Bytes(Vec<u8>),
    /// Property is a signed integer.
    SInt(i32),
    /// Property is an unsigned integer.
    UInt(u32),
    /// Property is a float.
    Float(f64),
    // /// Dynamically typed value.
    // Any(Box<dyn Any>),
}

// pub enum PropertyKey {
//     String(String),
//
// }

impl From<String> for PropertyValue {
    fn from(v: String) -> Self {
        PropertyValue::String(Rc::new(v))
    }
}

impl From<Rc<String>> for PropertyValue {
    fn from(v: Rc<String>) -> Self {
        PropertyValue::String(v)
    }
}

impl From<&Rc<String>> for PropertyValue {
    fn from(v: &Rc<String>) -> Self {
        PropertyValue::String(v.clone())
    }
}

impl From<&str> for PropertyValue {
    fn from(v: &str) -> Self {
        PropertyValue::String(Rc::new(v.to_string()))
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
pub struct PropertyStore<K>
    where K: Hash + Eq {
    content: HashMap<K, PropertyValue>
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
            content: HashMap::new()
        }
    }

    /// Insert a property.
    /// Returns the old property value if there was already a property stored under this key.
    pub fn insert<V: Into<PropertyValue>>(&mut self, key: K, value: V) -> Option<PropertyValue> {
        self.content.insert(key, value.into())
    }

    /// Get a property value by the property key.
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&PropertyValue>
        where K: Borrow<Q>,
              Q: Eq + Hash {
        self.content.get(key)
    }

    /// Check if the `key` is contained in this property store.
    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
        where K: Borrow<Q>,
              Q: Eq + Hash {
        self.content.contains_key(key)
    }

    /// Get a string property value by key.
    /// If the property value is not a string `None` is returned.
    pub fn get_string<Q: ?Sized>(&self, key: &Q) -> Option<&Rc<String>>
        where K: Borrow<Q>,
              Q: Eq + Hash {
        self.get(key)
            .and_then(|v| if let PropertyValue::String(s) = v {
                Some(s)
            } else {
                None
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
        where F: FnOnce(Option<&PropertyStore<Self::Key>>) -> R;

    /// Get mutable reference to the property storage.
    fn with_properties_mut<F, R>(&self, f: F) -> R
        where F: FnOnce(&mut PropertyStore<Self::Key>) -> R;

    /// Get a property value by the property key.
    fn property<Q: ?Sized>(&self, key: &Q) -> Option<PropertyValue>
        where Self::Key: Borrow<Q>,
              Q: Eq + Hash {
        self.with_properties(|p|
            p.and_then(|p| p.get(key).cloned())
        )
    }

    /// Get a string property value by key.
    /// If the property value is not a string `None` is returned.
    fn property_str<Q: ?Sized>(&self, key: &Q) -> Option<Rc<String>>
        where Self::Key: Borrow<Q>,
              Q: Eq + Hash {
        self.with_properties(|p|
            p.and_then(|p| p.get_string(key).cloned())
        )
    }

    /// Insert a property.
    /// Returns the old property value if there was already a property stored under this key.
    fn set_property<V: Into<PropertyValue>>(&self, key: Self::Key, value: V) -> Option<PropertyValue> {
        self.with_properties_mut(|p| p.insert(key, value))
    }
}