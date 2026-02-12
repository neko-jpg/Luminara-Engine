use crate::app::App;

pub trait Plugin: Send + Sync + 'static {
    fn build(&self, app: &mut App);
    fn name(&self) -> &str;
}
