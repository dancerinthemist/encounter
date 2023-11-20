use crate::action::{Modifier, ValueChange};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Character {
    name: String,
    pub(crate) attributes: Vec<Attribute>,
    attribute_map: HashMap<String, usize>,
    attribute_modifiers: Vec<Vec<Modifier>>,
}

impl Character {
    pub fn new(name: &str, attributes: Vec<Attribute>) -> Character {
        let attribute_map = attributes
            .iter()
            .enumerate()
            .map(|(i, a)| (a.name.to_string(), i))
            .collect::<_>();
        let modifiers = vec![vec![]; attributes.len()];
        Self {
            name: name.to_string(),
            attributes,
            attribute_map,
            attribute_modifiers: modifiers,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn add_attribute(&mut self, attribute: Attribute) {
        let name = attribute.name.clone();
        self.attributes.push(attribute);
        self.attribute_map.insert(name, self.attributes.len() - 1);
        self.attribute_modifiers.push(vec![]);
    }

    pub fn add_modifier(&mut self, for_value: &str, modifier: Modifier) {
        if let Some(i) = self.attribute_map.get(&for_value.to_ascii_lowercase()) {
            if let Some(mods) = self.attribute_modifiers.get_mut(*i) {
                mods.push(modifier);
            }
        }
    }

    pub(crate) fn apply(&mut self, change: &ValueChange) {
        if let Some(i) = self.attribute_map.get(change.name()) {
            if let Some(a) = self.attributes.get_mut(*i) {
                change.apply_with_modifiers(a, self.attribute_modifiers.get(*i).unwrap_or(&vec![]));
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Attribute {
    name: String,
    pub(crate) value: f64,
}

impl Attribute {
    pub(crate) fn set_value(&mut self, new_value: f64) {
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
        }
    }

    pub fn with_value(self, value: f64) -> Self {
        let mut result = self;
        result.value = value;
        result
    }
}
