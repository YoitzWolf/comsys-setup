use std::fmt::Debug;




pub trait ConvertablePackage: From<( std::string::String, Vec<Self::Item> )> + Clone + Debug {
    type Item : TryFrom<(i32, Vec<String>)> + Clone + Debug; 
}