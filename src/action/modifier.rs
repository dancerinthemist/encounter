use crate::action::AttributeStatusCollection;
use crate::{Action, Attribute, AttributeChange, Status, StatusChange};
use dyn_clone::DynClone;
use std::collections::HashMap;
use std::fmt::Debug;

pub trait Modifier: Default + Debug + DynClone {
    type Attr: Attribute;

    fn apply(&self, on: <Self::Attr as Attribute>::Value) -> <Self::Attr as Attribute>::Value {
        on
    }

    fn apply_if_applicable<S>(
        &self,
        on: <Self::Attr as Attribute>::Value,
        _attribute_change: Option<&AttributeChange<Self::Attr>>,
        _status_change: Option<&StatusChange<S>>,
        _actor: &AttributeStatusCollection<Self::Attr, S>,
        _receiver: &AttributeStatusCollection<Self::Attr, S>,
        _action: &Action<Self::Attr, S>,
    ) -> <Self::Attr as Attribute>::Value
    where
        S: Status,
    {
        on
    }
}

pub type IncomingModifierCollection<M> = ModifierCollection<M>;
pub type OutgoingModifierCollection<M> = ModifierCollection<M>;

#[derive(Default, Debug, Clone)]
pub struct ModifierCollection<M: Modifier> {
    by_attribute_name: HashMap<<M::Attr as Attribute>::Identifier, Vec<M>>,
    by_action_name: HashMap<String, Vec<M>>,
    complex: Vec<Box<M>>,
}

impl<M: Modifier> ModifierCollection<M> {
    pub(crate) fn generate_attribute_change<S>(
        &self,
        attribute_change: &AttributeChange<M::Attr>,
        actor: &AttributeStatusCollection<M::Attr, S>,
        receiver: &AttributeStatusCollection<M::Attr, S>,
        action: &Action<M::Attr, S>,
    ) -> AttributeChange<M::Attr>
    where
        S: Status,
    {
        let mut result = attribute_change.clone();
        if let Some(v) = self.by_attribute_name.get(attribute_change.identifier()) {
            v.iter()
                .for_each(|x| result.change = x.apply(result.change));
        }
        if let Some(v) = self.by_action_name.get(action.name()) {
            v.iter()
                .for_each(|x| result.change = x.apply(result.change));
        }
        self.complex.iter().for_each(|x| {
            result.change = x.apply_if_applicable(
                result.change,
                Some(attribute_change),
                None,
                actor,
                receiver,
                action,
            )
        });
        result
    }
}
