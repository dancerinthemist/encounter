use crate::{Attribute, Character, Error};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::any::Any;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub enum Modifier {
    Add(f64),
    Mul(f64),
}

impl Default for Modifier {
    fn default() -> Self {
        Self::Add(0.)
    }
}

impl Modifier {
    pub(crate) fn apply(&self, value: &mut f64) {
        match self {
            Self::Add(v) => *value += v,
            Self::Mul(v) => *value *= v,
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Target {
    Actor,
    Target,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValueChange {
    name: String,
    change: f64,
}

impl ValueChange {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl ValueChange {
    pub(crate) fn apply_with_modifiers(
        &self,
        attribute: &mut Attribute,
        modifiers: &Vec<Modifier>,
    ) {
        attribute.set_value(attribute.value() + self.apply_modifiers(modifiers));
    }

    fn apply_modifiers(&self, modifiers: &Vec<Modifier>) -> f64 {
        let mut applied = self.change;
        for m in modifiers {
            m.apply(&mut applied);
        }
        applied
    }

    pub fn new(name: &str, change: f64) -> Self {
        Self {
            name: name.to_ascii_lowercase(),
            change,
        }
    }
}

pub struct ActionOutput;

pub struct ActionInfo;

impl ActionInfo {
    pub(crate) fn name(&self) -> &str {
        todo!()
    }
}

pub trait Action: Debug + ActionClone + AsAny {
    fn apply(&self, characters: &mut [Character], actor: usize, targets: &[usize]) -> ActionOutput;

    fn name(&self) -> Option<&String>;
}

pub trait ActionClone {
    fn clone_box(&self) -> Box<dyn Action>;
}

impl<T> ActionClone for T
where
    T: 'static + Action + Clone,
{
    fn clone_box(&self) -> Box<dyn Action> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Action> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleAction {
    name: Option<String>,
    target: Target,
    changes: Vec<ValueChange>,
}

impl SimpleAction {
    pub fn new(name: &str, target: Target, changes: Vec<ValueChange>) -> Self {
        Self {
            name: Some(name.to_string()),
            target,
            changes,
        }
    }

    pub fn new_anonymous(target: Target, changes: Vec<ValueChange>) -> Self {
        Self {
            name: None,
            target,
            changes,
        }
    }
}

impl Action for SimpleAction {
    fn apply(&self, characters: &mut [Character], actor: usize, targets: &[usize]) -> ActionOutput {
        let t = &self.target;
        let changes = &self.changes;
        for change in changes {
            let targets_for_action = match t {
                Target::Actor => {
                    vec![actor]
                }
                Target::Target => {
                    vec![*targets.first().unwrap()]
                }
            };
            for t in targets_for_action {
                if let Some(c) = characters.get_mut(t) {
                    c.apply(change);
                }
            }
        }
        ActionOutput
    }

    fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct ComplexAction {
    name: Option<String>,
    actions: Vec<Box<dyn Action>>,
}

impl ComplexAction {
    pub fn new(name: Option<String>, actions: Vec<Box<dyn Action>>) -> Self {
        Self { name, actions }
    }
}

impl Action for ComplexAction {
    fn apply(&self, characters: &mut [Character], actor: usize, targets: &[usize]) -> ActionOutput {
        for a in &self.actions {
            a.as_ref().apply(characters, actor, targets);
        }
        ActionOutput
    }

    fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }
}

pub enum DeserializedAction {
    Complex(ComplexAction),
    Simple(SimpleAction),
}

impl DeserializedAction {
    pub(crate) fn boxed(self) -> Box<dyn Action> {
        match self {
            Self::Simple(s) => Box::new(s),
            Self::Complex(c) => Box::new(c),
        }
    }

    pub fn unwrap_simple(self) -> SimpleAction {
        match self {
            Self::Simple(s) => s,
            _ => panic!("Called unwrap_simple() on non-simple action"),
        }
    }

    pub fn unwrap_complex(self) -> ComplexAction {
        match self {
            Self::Complex(c) => c,
            _ => panic!("Called unwrap_complex() on non-complex action"),
        }
    }
}

pub fn deserialize_action_from_str(str: &str) -> Result<DeserializedAction, Error> {
    let value: Value = serde_json::from_str(str).map_err(|_| Error::ToDo)?;
    deserialize_action(value, true)
}

pub fn deserialize_action(value: Value, require_name: bool) -> Result<DeserializedAction, Error> {
    if is_simple_action(&value) {
        let r = deserialize_simple_action(value).map_err(|_| Error::ToDo)?;
        if require_name && r.name.is_none() {
            Err(Error::ToDo)
        } else {
            Ok(DeserializedAction::Simple(r))
        }
    } else {
        if let Some(o) = value.as_object() {
            let name = o
                .get("name")
                .and_then(|v| v.as_str().map(|s| s.to_string()));
            if require_name && name.is_none() {
                return Err(Error::ToDo);
            }
            if let Some(m) = o.get("actions").and_then(|v| v.as_array()) {
                let inner_actions = m
                    .iter()
                    .map(|v| deserialize_action(v.clone(), false))
                    .map(|v| v.map(|d| d.boxed()))
                    .collect::<Result<Vec<Box<dyn Action>>, _>>()?;
                return Ok(DeserializedAction::Complex(ComplexAction::new(
                    name,
                    inner_actions,
                )));
            }
        }
        Err(Error::ToDo)
    }
}

trait UnwrapBoxedAction {
    fn as_simple(&self) -> Option<SimpleAction>;
    fn as_complex(&self) -> Option<ComplexAction>;
}

impl UnwrapBoxedAction for Box<dyn Action> {
    fn as_simple(&self) -> Option<SimpleAction> {
        self.as_ref().as_any().downcast_ref::<_>().cloned()
    }

    fn as_complex(&self) -> Option<ComplexAction> {
        self.as_ref().as_any().downcast_ref::<_>().cloned()
    }
}

fn is_simple_action(value: &Value) -> bool {
    if let Some(o) = value.as_object() {
        o.contains_key("target") && o.contains_key("changes") && !o.contains_key("actions")
    } else {
        false
    }
}

fn deserialize_simple_action(value: Value) -> Result<SimpleAction, serde_json::Error> {
    serde_json::from_value(value)
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
mod tests {
    use super::*;
    use crate::Attribute;

    fn create_scene() -> (Vec<Character>, (SimpleAction, SimpleAction), Vec<usize>) {
        let attributes = vec![Attribute::new("rage")];
        let c1 = Character::new("C1", attributes.clone());
        let c2 = Character::new("C2", attributes.clone());
        let characters = vec![c1, c2];
        let action1 =
            SimpleAction::new_anonymous(Target::Actor, vec![ValueChange::new("rage", 5.0)]);
        let action2 =
            SimpleAction::new_anonymous(Target::Target, vec![ValueChange::new("rage", 5.0)]);
        let targets = vec![1];
        (characters, (action1, action2), targets)
    }

    #[test]
    fn test_action() {
        let (mut characters, (action1, action2), targets) = create_scene();
        action1.apply(&mut characters, 0, &targets);
        assert_eq!(characters[0].attributes[0].value, 5.);
        assert_eq!(characters[1].attributes[0].value, 0.);
        action2.apply(&mut characters, 0, &targets);
        assert_eq!(characters[0].attributes[0].value, 5.);
        assert_eq!(characters[1].attributes[0].value, 5.);
        action1.apply(&mut characters, 1, &targets);
        assert_eq!(characters[0].attributes[0].value, 5.);
        assert_eq!(characters[1].attributes[0].value, 10.);
    }

    #[test]
    fn test_modifiers() {
        let (mut characters, (action1, action2), targets) = create_scene();
        characters[0].add_modifier(&action1.changes[0].name, Modifier::Add(3.));
        action1.apply(&mut characters, 0, &targets);
        assert_eq!(characters[0].attributes[0].value, 8.);
        characters[1].add_modifier(&action1.changes[0].name, Modifier::Mul(0.2));
        action2.apply(&mut characters, 0, &targets);
        assert_eq!(characters[1].attributes[0].value, 1.);
    }

    #[test]
    fn test_deserialize() {
        let str = r#"
            {
                "name" : "SimpleAction",
                "target" : "Target",
                "changes" : [ {
                    "name" : "a",
                    "change" : 1
                }]
            }
        "#;
        let result = deserialize_action_from_str(str).unwrap().unwrap_simple();

        assert_eq!(result.name, Some("SimpleAction".to_string()));
        assert_eq!(result.changes, vec![ValueChange::new("a", 1.)]);
        let str = r#"{
            "name" : "ComplexAction",
            "actions" : [
                {
                    "target" : "Target",
                    "changes" : []
                },
                {
                    "name" : "InnerComplex",
                    "actions" : []
                }
            ]
        }"#;
        let result = deserialize_action_from_str(str).unwrap().unwrap_complex();
        assert_eq!(result.name, Some("ComplexAction".to_string()));
        println!("{:?}", result.actions[0].type_id());
        println!("{:?}", result.actions[1].type_id());
        let s = result.actions[0].as_simple();
        let c = result.actions[1].as_complex();
        assert!(s.is_some());
        assert!(c.is_some());
    }
}
