use crate::world::{World, ComponentInfo};
use crate::component::Component;
use crate::change_detection::ComponentTicks;
use std::any::TypeId;
use std::collections::HashMap;

pub trait Bundle: Send + Sync + 'static {
    fn register_components(world: &mut World);
    fn get_components(self, info: &HashMap<TypeId, ComponentInfo>, ticks: ComponentTicks) -> HashMap<TypeId, (*const u8, ComponentTicks)>;
    fn component_ids() -> Vec<TypeId>;
}

impl Bundle for () {
    fn register_components(_: &mut World) {}
    fn get_components(self, _: &HashMap<TypeId, ComponentInfo>, _: ComponentTicks) -> HashMap<TypeId, (*const u8, ComponentTicks)> {
        HashMap::new()
    }
    fn component_ids() -> Vec<TypeId> { Vec::new() }
}

impl<T: Component> Bundle for T {
    fn register_components(world: &mut World) {
        world.register_component::<T>();
    }
    fn get_components(self, _info: &HashMap<TypeId, ComponentInfo>, ticks: ComponentTicks) -> HashMap<TypeId, (*const u8, ComponentTicks)> {
        let mut map = HashMap::new();
        let ptr = Box::into_raw(Box::new(self)) as *const u8;
        map.insert(TypeId::of::<T>(), (ptr, ticks));
        map
    }
    fn component_ids() -> Vec<TypeId> { vec![TypeId::of::<T>()] }
}

// Implement for tuples
macro_rules! impl_bundle_tuple {
    ($($t:ident),*) => {
        impl<$($t: Component),*> Bundle for ($($t,)*) {
            fn register_components(world: &mut World) {
                $(world.register_component::<$t>();)*
            }
            fn get_components(self, _info: &HashMap<TypeId, ComponentInfo>, ticks: ComponentTicks) -> HashMap<TypeId, (*const u8, ComponentTicks)> {
                #[allow(non_snake_case)]
                let ($($t,)*) = self;
                let mut map = HashMap::new();
                $(
                    let ptr = Box::into_raw(Box::new($t)) as *const u8;
                    map.insert(TypeId::of::<$t>(), (ptr, ticks));
                )*
                map
            }
            fn component_ids() -> Vec<TypeId> {
                vec![$(TypeId::of::<$t>()),*]
            }
        }
    };
}

impl_bundle_tuple!(A, B);
impl_bundle_tuple!(A, B, C);
impl_bundle_tuple!(A, B, C, D);
impl_bundle_tuple!(A, B, C, D, E);
