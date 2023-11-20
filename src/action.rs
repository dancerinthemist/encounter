use crate::{Attribute, Character};

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

#[derive(Debug, Copy, Clone)]
pub enum Target {
    Actor,
    Target,
}

#[derive(Debug, Clone)]
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

pub trait Action {
    fn apply(&self, characters: &mut [Character], actor: usize, targets: &[usize]) -> ActionOutput;

    fn name(&self) -> Option<&String>;
}
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

pub struct ComplexAction {
    name: Option<String>,
    actions: Vec<Box<dyn Action>>,
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
}
