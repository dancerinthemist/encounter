use crate::{ActionInfo, Character, ValueChange};
use std::collections::HashMap;

pub struct Modifier {}

pub struct ModifierCollection {
    by_action_name: HashMap<String, Modifier>,
}

impl ModifierCollection {
    pub fn apply(
        &self,
        value: f64,
        value_change: &ValueChange,
        action_info: &ActionInfo,
        characters: &[&Character],
        actor: usize,
        targets: &[usize],
    ) -> f64 {
        self.apply_complex(
            self.apply_characters(
                self.apply_action_info(self.apply_value_change(value, value_change), action_info),
                characters,
                actor,
                targets,
            ),
            value_change,
            action_info,
            characters,
            actor,
            targets,
        )
    }
    fn apply_value_change(&self, value: f64, value_change: &ValueChange) -> f64 {
        todo!()
    }
    fn apply_characters(
        &self,
        value: f64,
        characters: &[&Character],
        actor: usize,
        targets: &[usize],
    ) -> f64 {
        todo!()
    }
    fn apply_action_info(&self, value: f64, action_info: &ActionInfo) -> f64 {
        todo!()
    }
    fn apply_complex(
        &self,
        value: f64,
        value_change: &ValueChange,
        action_info: &ActionInfo,
        characters: &[&Character],
        actor: usize,
        targets: &[usize],
    ) -> f64 {
        todo!()
    }
}
