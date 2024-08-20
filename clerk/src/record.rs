
pub trait KeyMapper: Default {

    fn keys() -> Vec<String>;
    fn to_key(&self) -> String;
    //fn from_key(key: &str) -> Self;
}

pub trait Record {
    type KeyMap: KeyMapper;
    fn change_key(&mut self, v: Self::KeyMap) -> Result<(), ()>;
    fn change_key_vec(&mut self, vs: Vec<Self::KeyMap>) -> Result<usize, ()>;
    fn as_line(&self) -> Vec<String>;
    fn from_line(line: Vec<String>) -> Result<Self, ()> where Self: Sized;
}