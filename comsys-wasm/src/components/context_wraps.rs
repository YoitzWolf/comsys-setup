use yew::Reducible;

pub enum DropSetAction<T>  {
    Set(T),
    Drop
}


#[derive(Debug, Clone, Default, PartialEq)]
pub struct IdListContext(pub Vec<i32>);

impl Reducible for IdListContext {
    type Action = DropSetAction<IdListContext>;

    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        todo!()
    }
}