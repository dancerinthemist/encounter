pub(crate) mod modifier;
pub mod output;

use crate::action::modifier::{IncomingModifierCollection, OutgoingModifierCollection};
use crate::{AttributeCollection, Status, StatusCollection};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
pub struct Action {
    name: String,
    inner: InnerAction,
}

impl Action {
    pub fn new(name: String, inner: InnerAction) -> Self {
        Self { name, inner }
    }
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn inner(&self) -> &InnerAction {
        &self.inner
    }

    pub fn apply_modifiers(&self, actor: &Actor, receiver: &Receiver) -> InnerAction {
        self.inner.apply_modifiers(actor, receiver, self)
    }
}

#[derive(Debug)]
pub enum InnerAction {
    Simple(SimpleAction),
    SelfOther(SimpleAction, SimpleAction),
    Custom(Box<dyn CustomAction>),
}

impl InnerAction {
    pub fn apply_modifiers(&self, actor: &Actor, receiver: &Receiver, action: &Action) -> Self {
        match self {
            Self::Simple(a) => Self::Simple(a.apply_modifiers(actor, receiver, action)),
            Self::SelfOther(a1, a2) => Self::SelfOther(
                a1.apply_modifiers(actor, actor, action),
                a2.apply_modifiers(actor, receiver, action),
            ),
            _ => todo!(),
        }
    }

    pub fn apply(
        &self,
        attribute_collections: &mut [&mut AttributeCollection],
        status_collections: &mut [&mut StatusCollection],
        targets: &HashMap<Target, usize>,
    ) {
        match self {
            InnerAction::Simple(a) => {
                a.apply(
                    *attribute_collections
                        .get_mut(targets[&Target::Target])
                        .unwrap(),
                    *status_collections
                        .get_mut(targets[&Target::Target])
                        .unwrap(),
                );
            }
            InnerAction::SelfOther(a1, a2) => {
                a1.apply(
                    *attribute_collections
                        .get_mut(targets[&Target::Actor])
                        .unwrap(),
                    *status_collections.get_mut(targets[&Target::Actor]).unwrap(),
                );
                a2.apply(
                    *attribute_collections
                        .get_mut(targets[&Target::Target])
                        .unwrap(),
                    *status_collections
                        .get_mut(targets[&Target::Target])
                        .unwrap(),
                );
            }
            InnerAction::Custom(_) => todo!(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleAction {
    target: Target,
    elements: Vec<ActionElement>,
}

impl SimpleAction {
    pub fn new_empty(target: Target) -> Self {
        Self::new(target, vec![])
    }
    pub fn new(target: Target, elements: Vec<ActionElement>) -> Self {
        Self { target, elements }
    }

    pub fn set_target(&mut self, target: Target) {
        self.target = target;
    }

    pub fn apply_modifiers(
        &self,
        actor: &Actor,
        receiver: &Receiver,
        action: &Action,
    ) -> SimpleAction {
        let mut result = SimpleAction::new_empty(self.target);
        for e in &self.elements {
            match e {
                ActionElement::AttributeChange(a) => result.elements.push(
                    ActionElement::AttributeChange(a.apply_modifiers(actor, receiver, action)),
                ),
                other => result.elements.push(other.clone()),
            }
        }
        result
    }

    pub fn apply(&self, attributes: &mut AttributeCollection, statuses: &mut StatusCollection) {
        for e in &self.elements {
            e.apply(attributes, statuses);
        }
    }
}

pub type Actor<'a> = (
    &'a AttributeCollection,
    &'a StatusCollection,
    &'a OutgoingModifierCollection,
);
pub type Receiver<'a> = (
    &'a AttributeCollection,
    &'a StatusCollection,
    &'a IncomingModifierCollection,
);

// ===============
// Custom Actions
// ===============

pub trait CustomAction: Debug + ActionClone + AsAny {
    fn name(&self) -> Option<&String>;
}

pub trait ActionClone {
    fn clone_box(&self) -> Box<dyn CustomAction>;
}

impl<T> ActionClone for T
where
    T: 'static + CustomAction + Clone,
{
    fn clone_box(&self) -> Box<dyn CustomAction> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn CustomAction> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

// ===============
// Target & Changes
// ===============

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialOrd, PartialEq, Eq, Hash)]
pub enum Target {
    Actor,
    Target,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ActionElement {
    AttributeChange(AttributeChange),
    StatusChange(StatusChange),
}

impl ActionElement {
    pub(crate) fn apply(
        &self,
        attributes: &mut AttributeCollection,
        statuses: &mut StatusCollection,
    ) {
        match self {
            ActionElement::AttributeChange(a) => a.apply(attributes),
            ActionElement::StatusChange(s) => s.apply(statuses),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum AttributeChangeType {
    Add,
    Mul,
    Set,
    Average(f64, f64), // Weights
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AttributeChange {
    name: String,
    change: f64,
    op: AttributeChangeType,
}

impl Into<ActionElement> for AttributeChange {
    fn into(self) -> ActionElement {
        ActionElement::AttributeChange(self)
    }
}

impl AttributeChange {
    pub(crate) fn apply(&self, attributes: &mut AttributeCollection) {
        if let Some(a) = attributes.get_attribute_mut(self.name()) {
            match self.op {
                AttributeChangeType::Add => a.set_value(a.value + self.change),
                AttributeChangeType::Mul => a.set_value(a.value * self.change),
                AttributeChangeType::Set => a.set_value(self.change),
                AttributeChangeType::Average(weight_current, weight_new) => {
                    a.set_value((a.value * weight_current + self.change * weight_new)
                        / (weight_current + weight_new));
                }
            }
        }
    }
}

pub(crate) type AttributeStatusCollection<'a> = (&'a AttributeCollection, &'a StatusCollection);

impl AttributeChange {
    pub(crate) fn apply_modifiers(
        &self,
        actor: &Actor,
        receiver: &Receiver,
        action: &Action,
    ) -> Self {
        let (actor_attributes, actor_statuses, actor_collection) = actor;
        let (receiver_attributes, receiver_statuses, receiver_collection) = receiver;
        let actor = (*actor_attributes, *actor_statuses);
        let receiver = (*receiver_attributes, *receiver_statuses);
        receiver_collection.generate_attribute_change(
            &actor_collection.generate_attribute_change(self, &actor, &receiver, action),
            &actor,
            &receiver,
            action,
        )
    }
}

impl AttributeChange {
    pub fn new(name: &str, change: f64) -> Self {
        Self {
            name: name.to_ascii_lowercase(),
            change,
            op : AttributeChangeType::Add
        }
    }

    pub fn with_op(mut self, op: AttributeChangeType) -> Self {
        self.op = op;
        self
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

/// Describes Changing a Status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatusChange {
    Add(Status),
    Remove(Status),
}

impl StatusChange {
    pub(crate) fn apply(&self, statuses: &mut StatusCollection) {
        todo!()
    }
}

// Extension trait for downcasting
pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

impl<T: 'static> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {}
