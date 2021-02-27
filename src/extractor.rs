pub trait Extractor<C, R> {
    fn extract(&self, text: &str, ctx: &C) -> Vec<R>;
}
