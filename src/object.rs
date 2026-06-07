use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ObjString(pub String);

impl ObjString {
    pub fn new(s: String) -> Rc<Self> {
        Rc::new(ObjString(s))
    }
}
