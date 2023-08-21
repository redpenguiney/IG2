pub trait GameObject {

}

#[macro_export]
macro_rules! impl_gameobject {
    ($structname: ident) => {
        impl crate::GameObject for $structname {
            
        }
    }
}