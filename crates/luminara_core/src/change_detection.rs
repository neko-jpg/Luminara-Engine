#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tick(pub u32);

impl Tick {
    pub fn increment(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ComponentTicks {
    pub added: Tick,
    pub changed: Tick,
}
