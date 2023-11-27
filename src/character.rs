use crate::action::modifier::{IncomingModifierCollection, OutgoingModifierCollection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ===============
// Character
// ===============

pub type Character = (
    CharacterBase,
    AttributeCollection,
    StatusCollection,
    IncomingModifierCollection,
    OutgoingModifierCollection,
);

pub struct CharacterBase {
    name: String,
}

impl CharacterBase {
    pub(crate) fn name(&self) -> &str {
        self.name.as_ref()
    }
}

impl CharacterBase {
    pub fn new(name: &str) -> Character {
        (
            Self::new_base(name),
            AttributeCollection::default(),
            StatusCollection::default(),
            IncomingModifierCollection::default(),
            OutgoingModifierCollection::default(),
        )
    }
    fn new_base(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

// ===============
// Attribute
// ===============

#[derive(Default, Debug)]
pub struct AttributeCollection {
    attributes: Vec<Attribute>,
    attribute_map: HashMap<String, usize>,
}

impl AttributeCollection {
    pub fn new() -> Self {
        Self {
            attributes: vec![],
            attribute_map: HashMap::new(),
        }
    }

    pub fn add_attribute(&mut self, attribute: Attribute) {
        self.attribute_map
            .insert(attribute.name.clone(), self.attributes.len());
        self.attributes.push(attribute);
    }

    pub fn get_attribute(&self, name: &str) -> Option<&Attribute> {
        self.attribute_map
            .get(name)
            .and_then(|idx| self.attributes.get(*idx))
    }

    pub fn get_attribute_value(&self, name: &str) -> Option<f64> {
        self.get_attribute(name).map(|a| a.value)
    }

    pub fn get_attribute_mut(&mut self, name: &str) -> Option<&mut Attribute> {
        self.attribute_map
            .get(name)
            .and_then(|idx| self.attributes.get_mut(*idx))
    }

    pub fn set_attribute_value(&mut self, name: &str, value: f64) {
        if let Some(a) = self.get_attribute_mut(name) {
            a.set_value(value);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Attribute {
    name: String,
    pub(crate) value: f64,
    min: Option<f64>,
    max: Option<f64>
}

impl Attribute {
    pub(crate) fn set_value(&mut self, new_value: f64) {
        let new_value = self.max.map_or(new_value, |m| new_value.min(m));
        let new_value = self.min.map_or(new_value, |m| new_value.max(m));
        self.value = new_value;
    }

    pub fn value(&self) -> f64 {
        self.value
    }
}

impl Attribute {
    pub fn new(name: &str) -> Self {
        Attribute {
            name: name.to_ascii_lowercase(),
            value: 0.,
            min: None,
            max: None,
        }
    }

    pub fn with_value(mut self, value: f64) -> Self {
        self.value = value;
        self
    }

    pub fn with_bounds(mut self, min: Option<f64>, max: Option<f64>) -> Self {
        self.min = min;
        self.max = max;
        self
    }
}

// ===============
// STATUS
// ===============

#[derive(Default, Debug)]
pub struct StatusCollection;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Status {
    TODO,
}
