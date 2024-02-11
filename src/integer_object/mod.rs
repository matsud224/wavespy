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
    pub fn new(number: i32) -> Self {
        Object::builder().property("number", number).build()
    }

    pub fn add_child(&self, i: i32) {
        self.imp().children.borrow_mut().push(i);
    }

    pub fn children(&self) -> &Vec<i32> {
        self.imp().children.borrow().as_ref()
    }
}

// ANCHOR_END: integer_object
