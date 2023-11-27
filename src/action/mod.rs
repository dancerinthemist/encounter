pub(crate) mod modifier;
pub mod output;

use crate::action::modifier::{IncomingModifierCollection, Modifier, OutgoingModifierCollection};
use crate::{Attribute, AttributeCollection, AttributeValue, Status, StatusCollection};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;

pub struct Action<A: Attribute, S: Status> {
    name: String,
    inner: InnerAction<A, S>,
}

impl<A: Attribute, S: Status> Action<A, S> {
    pub fn new(name: String, inner: InnerAction<A, S>) -> Self {
        Self { name, inner }
    }
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn inner(&self) -> &InnerAction<A, S> {
        &self.inner
    }

    pub fn apply_modifiers<M>(
        &self,
        actor: &Actor<A, S, M>,
        receiver: &Receiver<A, S, M>,
    ) -> InnerAction<A, S>
    where
        M: Modifier<Attr = A>,
    {
        self.inner.apply_modifiers(actor, receiver, self)
    }
}

#[derive(Debug)]
pub enum InnerAction<A: Attribute, S: Status> {
    Simple(SimpleAction<A, S>),
    SelfOther(SimpleAction<A, S>, SimpleAction<A, S>),
    Custom(Box<dyn CustomAction>),
}

impl<A: Attribute, S: Status> InnerAction<A, S> {
    pub fn apply_modifiers<M>(
        &self,
        actor: &Actor<A, S, M>,
        receiver: &Receiver<A, S, M>,
        action: &Action<A, S>,
    ) -> Self
    where
        M: Modifier<Attr = A>,
    {
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
        attribute_collections: &mut [&mut AttributeCollection<A>],
        status_collections: &mut [&mut StatusCollection<S>],
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

#[derive(Debug, Clone)]
pub struct SimpleAction<A: Attribute, S: Status> {
    target: Target,
    elements: Vec<ActionElement<A, S>>,
}

impl<A: Attribute, S: Status> SimpleAction<A, S> {
    pub fn new_empty(target: Target) -> Self {
        Self::new(target, vec![])
    }
    pub fn new(target: Target, elements: Vec<ActionElement<A, S>>) -> Self {
        Self { target, elements }
    }

    pub fn set_target(&mut self, target: Target) {
        self.target = target;
    }

    pub fn apply_modifiers<M>(
        &self,
        actor: &Actor<A, S, M>,
        receiver: &Receiver<A, S, M>,
        action: &Action<A, S>,
    ) -> SimpleAction<A, S>
    where
        M: Modifier<Attr = A>,
    {
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

    pub fn apply(
        &self,
        attributes: &mut AttributeCollection<A>,
        statuses: &mut StatusCollection<S>,
    ) {
        for e in &self.elements {
            e.apply(attributes, statuses);
        }
    }
}

pub type Actor<'a, A, S, M> = (
    &'a AttributeCollection<A>,
    &'a StatusCollection<S>,
    &'a OutgoingModifierCollection<M>,
);
pub type Receiver<'a, A, S, M> = (
    &'a AttributeCollection<A>,
    &'a StatusCollection<S>,
    &'a IncomingModifierCollection<M>,
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

#[derive(Clone, Debug)]
pub enum ActionElement<A: Attribute, S: Status> {
    AttributeChange(AttributeChange<A>),
    StatusChange(StatusChange<S>),
}

impl<A: Attribute, S: Status> ActionElement<A, S> {
    pub(crate) fn apply(
        &self,
        attributes: &mut AttributeCollection<A>,
        statuses: &mut StatusCollection<S>,
    ) {
        match self {
            ActionElement::AttributeChange(a) => a.apply(attributes),
            ActionElement::StatusChange(s) => s.apply(statuses),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum AttributeChangeType<V: AttributeValue> {
    Add,
    Mul,
    Set,
    Average(V, V), // Weights
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AttributeChange<A: Attribute> {
    identifier: A::Identifier,
    change: A::Value,
    op: AttributeChangeType<A::Value>,
}

impl<S: Status, A: Attribute> Into<ActionElement<A, S>> for AttributeChange<A> {
    fn into(self) -> ActionElement<A, S> {
        ActionElement::AttributeChange(self)
    }
}

impl<A: Attribute> AttributeChange<A> {
    pub(crate) fn apply(&self, attributes: &mut AttributeCollection<A>) {
        if let Some(a) = attributes.get_attribute_mut(&self.identifier) {
            match &self.op {
                AttributeChangeType::Add => a.set_value(a.value() + self.change),
                AttributeChangeType::Mul => a.set_value(a.value() * self.change),
                AttributeChangeType::Set => a.set_value(self.change),
                AttributeChangeType::Average(weight_current, weight_new) => {
                    a.set_value(
                        (a.value() * *weight_current + self.change * *weight_new)
                            / (*weight_current + *weight_new),
                    );
                }
            }
        }
    }
}

pub(crate) type AttributeStatusCollection<'a, A, S> =
    (&'a AttributeCollection<A>, &'a StatusCollection<S>);

impl<A: Attribute> AttributeChange<A> {
    pub(crate) fn apply_modifiers<S, M>(
        &self,
        actor: &Actor<A, S, M>,
        receiver: &Receiver<A, S, M>,
        action: &Action<A, S>,
    ) -> Self
    where
        S: Status,
        M: Modifier<Attr = A>,
    {
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

    pub fn new(identifier: A::Identifier, change: A::Value) -> Self {
        Self {
            identifier,
            change,
            op: AttributeChangeType::Add,
        }
    }

    pub fn with_op(mut self, op: AttributeChangeType<A::Value>) -> Self {
        self.op = op;
        self
    }

    pub fn identifier(&self) -> &A::Identifier {
        &self.identifier
    }
}

/// Describes Changing a Status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatusChange<S: Status> {
    Add(S),
    Remove(S),
}

impl<S: Status> StatusChange<S> {
    pub(crate) fn apply(&self, statuses: &mut StatusCollection<S>) {
        match self {
            Self::Add(s) => statuses.add(s.clone()),
            Self::Remove(s) => statuses.remove(s),
        }
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
