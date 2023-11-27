use crate::action::AttributeStatusCollection;
use crate::{Action, AttributeChange, StatusChange};
use dyn_clone::DynClone;
use std::collections::HashMap;
use std::fmt::Debug;

pub trait Modifier: Debug + DynClone {
    fn apply(&self, on: f64) -> f64 {
        on
    }

    fn apply_if_applicable(
        &self,
        on: f64,
        _attribute_change: Option<&AttributeChange>,
        _status_change: Option<&StatusChange>,
        _actor: &AttributeStatusCollection,
        _receiver: &AttributeStatusCollection,
        _action: &Action,
    ) -> f64 {
        on
    }
}

// Magic:
dyn_clone::clone_trait_object!(Modifier);

pub type IncomingModifierCollection = ModifierCollection;
pub type OutgoingModifierCollection = ModifierCollection;

#[derive(Default, Debug, Clone)]
pub struct ModifierCollection {
    by_attribute_name: HashMap<String, Vec<Box<dyn Modifier>>>,
    by_action_name: HashMap<String, Vec<Box<dyn Modifier>>>,
    complex: Vec<Box<dyn Modifier>>,
}

impl ModifierCollection {
    pub(crate) fn generate_attribute_change(
        &self,
        attribute_change: &AttributeChange,
        actor: &AttributeStatusCollection,
        receiver: &AttributeStatusCollection,
        action: &Action,
    ) -> AttributeChange {
        let mut result = attribute_change.clone();
        if let Some(v) = self.by_attribute_name.get(attribute_change.name()) {
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
