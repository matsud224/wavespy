mod imp;

use glib::Object;
use gtk::glib::{
    self,
    subclass::types::{ObjectSubclassExt, ObjectSubclassIsExt},
};

// ANCHOR: integer_object
glib::wrapper! {
    pub struct IntegerObject(ObjectSubclass<imp::IntegerObject>);
}

impl IntegerObject {
    pub fn new(number: i32, nums: &[i32]) -> Self {
        let iobj: Self = Object::builder().property("number", number).build();
        iobj.imp().children.borrow_mut().extend_from_slice(nums);
        iobj
    }

    pub fn add_nums(&self, nums: &[i32]) {}

    pub fn children(&self) -> Vec<i32> {
        self.imp().children.borrow().clone()
    }
}

// ANCHOR_END: integer_object
