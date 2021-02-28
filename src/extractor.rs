use std::fmt::Debug;

pub trait Extractor<C, R>: Send + Sync + Debug {
    fn extract(&self, text: &str, ctx: &C) -> Vec<R>;
}
