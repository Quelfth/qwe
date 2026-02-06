use std::{borrow::Borrow, collections::HashSet, hash::Hash};

#[derive(Debug)]
pub enum Pred {
    Name(String),
    Not(Box<Self>),
    And(Vec<Self>),
    Or(Vec<Self>),
}

impl Pred {
    pub fn parse(str: &str) -> Option<Self> {
        let disj = str.split("|").collect::<Vec<_>>();
        if disj.len() != 1 {
            return Some(Pred::Or(disj.into_iter().filter_map(Pred::parse).collect()));
        }
        let conj = str.split_whitespace().collect::<Vec<_>>();
        if conj.len() != 1 {
            return Some(Pred::And(
                conj.into_iter().filter_map(Pred::parse).collect(),
            ));
        }
        let mut not = false;
        let mut str = str;
        while let Some(neg) = str.trim().strip_prefix("!") {
            not = !not;
            str = neg
        }
        let pred = Pred::Name(str.trim().to_owned());

        Some(if not { Pred::Not(Box::new(pred)) } else { pred })
    }

    pub fn check(&self, values: &HashSet<impl Borrow<str> + Eq + Hash>) -> bool {
        match self {
            Pred::Name(name) => values.contains(name),
            Pred::Not(pred) => !pred.check(values),
            Pred::And(preds) => preds.iter().all(|p| p.check(values)),
            Pred::Or(preds) => preds.iter().any(|p| p.check(values)),
        }
    }
}
