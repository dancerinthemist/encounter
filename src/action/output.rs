use itertools::concat;
use crate::Character;

#[derive(Debug, Clone)]
pub struct ActionOutput {
    display_name: Option<String>,
    text: ActionText,
}

impl ActionOutput {
    pub(crate) fn new<T : ToString>(display_name: Option<&T>, text: &str) -> ActionOutput {
        Self {
            display_name : display_name.map(|x| x.to_string()),
            text : ActionText::Simple(text.to_string())
        }
    }

    pub fn combine(mut self, other: ActionOutput) -> ActionOutput {
        if self.display_name.is_none() {
            self.display_name = other.display_name;
        }
        
        self.text = self.text.combine(other.text);
        
        self

    }
}

#[derive(Debug, Clone)]
pub enum ActionText {
    Simple(String),
    Replace(String, Vec<Replace>),
    Multiple(Vec<Box<ActionText>>)
}

impl ActionText {
    pub(crate) fn combine(mut self, mut other: ActionText) -> Self {
        match (&mut self, &mut other) {
            (Self::Multiple(v), Self::Multiple(w)) => {
                Self::Multiple(concat(vec![v.clone(), w.clone()]))
            }
            (Self::Multiple(v), a) | (a, Self::Multiple(v)) => {
                v.push(Box::from(a.clone()));
                Self::Multiple(v.clone())
            }
            _ => { Self::Multiple(vec![Box::new(self), Box::new(other)])}
        }
    }
}

impl ActionText {
    pub fn format(&self, characters: &[Character], actor: usize, targets: &[usize]) -> String {
        match self {
            Self::Simple(s) => s.clone(),
            Self::Replace(s, r) => {
                let mut result = s.clone();
                for s in r {
                    let with = match s {
                        Replace::Actor => characters.get(actor).map(|x| x.name()),
                        Replace::Target => targets
                            .get(0)
                            .and_then(|i| characters.get(*i))
                            .map(|x| x.name()),
                        _ => None,
                    };
                    if let Some(w) = with {
                        result = result.replace(&s.to_string(), w);
                    }
                }
                result
            }
            Self::Multiple(v) => {
                v.iter().map(|a| a.format(characters, actor, targets)).collect::<Vec<String>>().join("\n")
            }
        }
    }

    pub fn extract_raw_string(&self) -> Option<&str> {
        match self {
            Self::Simple(s) | Self::Replace(s, _) => Some(s.as_str()),
            _ => None
        }
    }
}

#[derive(Debug, Clone)]
pub enum Replace {
    Actor,
    Target,
    Undefined
}

impl ToString for Replace {
    fn to_string(&self) -> String {
        match self {
            Replace::Actor => "{actor}".to_string(),
            Replace::Target => "{target}".to_string(),
            _ => String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format() {
        let initial_string = "aaa {target} bbb {actor}{target}";
        let text = ActionText::Replace(initial_string.to_string(), vec![Replace::Actor, Replace::Target]);
        let r = text.format(&[Character::new("ACTOR", vec![]), Character::new("TARGET", vec![])], 0, &[1]);
        assert_eq!("aaa TARGET bbb ACTORTARGET", r.as_str());
    }

    #[test]
    fn test_combine() {
        let a = ActionOutput::new(Some(&"a".to_string()), "text_a");
        let b = ActionOutput::new(Some(&"b".to_string()), "text_b");
        let ab = a.clone().combine(b.clone());
        assert_eq!(ab.display_name, Some("a".to_string()));
        let ba = b.clone().combine(a.clone());
        assert_eq!(ba.display_name, Some("b".to_string()));
    }
}