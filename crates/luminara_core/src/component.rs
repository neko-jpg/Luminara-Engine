pub trait Component: Send + Sync + 'static {
    fn type_name() -> &'static str
    where
        Self: Sized;
}

// Helper macro for implementing Component
#[macro_export]
macro_rules! impl_component {
    ($t:ty) => {
        impl $crate::component::Component for $t {
            fn type_name() -> &'static str {
                stringify!($t)
            }
        }
    };
}
