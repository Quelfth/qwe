use tree_sitter::{QueryPredicate, QueryPredicateArg};

use crate::pred::Pred;

#[derive(Debug)]
pub enum Predicate {
    Semantic { capture: u32, predicate: Pred },
}

pub struct PredicateError;

impl Predicate {
    pub fn parse(predicate: &QueryPredicate) -> Result<Self, PredicateError> {
        let QueryPredicate { operator, args } = predicate;
        match &**operator {
            "semantic?" => {
                let Some(QueryPredicateArg::Capture(capture)) = args.first() else {
                    return Err(PredicateError);
                };
                let mut pred = String::new();
                for arg in args.iter().skip(1) {
                    match arg {
                        QueryPredicateArg::Capture(_) => return Err(PredicateError),
                        QueryPredicateArg::String(s) => {
                            pred += s;
                            pred += " "
                        }
                    }
                }
                let Some(predicate) = Pred::parse(pred.trim()) else {
                    return Err(PredicateError);
                };

                Ok(Predicate::Semantic {
                    capture: *capture,
                    predicate,
                })
            }
            _ => Err(PredicateError),
        }
    }
}
