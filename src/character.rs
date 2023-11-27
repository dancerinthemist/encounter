use crate::action::modifier::{IncomingModifierCollection, Modifier, OutgoingModifierCollection};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{Add, Div, Mul, Sub};

// ===============
// Character
// ===============

pub type Character<A, S, M> = (
    CharacterBase,
    AttributeCollection<A>,
    StatusCollection<S>,
    IncomingModifierCollection<M>,
    OutgoingModifierCollection<M>,
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
    pub fn new<A, S, M>(name: &str) -> Character<A, S, M>
    where
        A: Attribute,
        S: Status,
        M: Modifier,
    {
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

pub trait AttributeIdentifier: Debug + Default + Clone + Hash + PartialEq + Eq {}

#[derive(Default, Debug)]
pub struct AttributeCollection<A: Attribute> {
    attributes: Vec<A>,
    attribute_map: HashMap<A::Identifier, usize>,
}

impl<A: Attribute> AttributeCollection<A> {
    pub fn new() -> Self {
        Self {
            attributes: vec![],
            attribute_map: HashMap::new(),
        }
    }

    pub fn add_attribute(&mut self, identifier: A::Identifier, attribute: A) {
        self.attribute_map.insert(identifier, self.attributes.len());
        self.attributes.push(attribute);
    }

    pub fn get_attribute(&self, identifier: &A::Identifier) -> Option<&A> {
        self.attribute_map
            .get(identifier)
            .and_then(|idx| self.attributes.get(*idx))
    }

    pub fn get_attribute_value(&self, identifier: &A::Identifier) -> Option<A::Value> {
        self.get_attribute(identifier).map(|a| a.value())
    }

    pub fn get_attribute_mut(&mut self, identifier: &A::Identifier) -> Option<&mut A> {
        self.attribute_map
            .get(identifier)
            .and_then(|idx| self.attributes.get_mut(*idx))
    }

    pub fn set_attribute_value(&mut self, identifier: &A::Identifier, value: A::Value) {
        if let Some(a) = self.get_attribute_mut(identifier) {
            a.set_value(value);
        }
    }
}

pub trait AttributeValue:
    Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Self, Output = Self>
    + Div<Self, Output = Self>
    + Debug
    + Copy
    + Clone
{
}

pub trait Attribute: Default + Debug + Clone {
    type Value: AttributeValue;
    type Identifier: AttributeIdentifier;
    fn set_value(&mut self, new_value: Self::Value);
    fn value(&self) -> Self::Value;
    fn with_value(mut self, value: Self::Value) -> Self {
        self.set_value(value);
        self
    }
}

// ===============
// STATUS
// ===============

#[derive(Default, Debug)]
pub struct StatusCollection<S: Status> {
    statuses: HashSet<S>,
}

impl<S: Status> StatusCollection<S> {
    pub fn add(&mut self, status: S) {
        self.statuses.insert(status);
    }

    pub fn remove(&mut self, status: &S) {
        self.statuses.remove(status);
    }
    pub fn contains(&self, status: &S) -> bool {
        self.statuses.contains(status)
    }
}

pub trait Status: Debug + Clone + Default + Eq + PartialEq + Hash {}
