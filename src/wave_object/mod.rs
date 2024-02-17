mod imp;
use crate::wave_viewer::WaveChangePoint;
use glib::Object;
use gtk::glib::property::PropertySet;
use gtk::glib::{self, subclass::types::ObjectSubclassIsExt};
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

glib::wrapper! {
    pub struct WaveObject(ObjectSubclass<imp::WaveObject>);
}

impl WaveObject {
    pub fn new(name: String, path: Vec<String>, data: Vec<WaveChangePoint>) -> Self {
        let obj: Self = Object::builder().build();
        obj.imp().data.set(WaveData {
            name,
            path,
            data,
            y_position: Cell::new(0.0),
            height: Cell::new(0.0),
        });
        obj
    }

    pub fn wave_data(&self) -> Rc<RefCell<WaveData>> {
        self.imp().data.clone()
    }

    pub fn set_y_position(&self, y: f32) {
        self.imp().data.borrow().y_position.set(y);
    }

    pub fn set_height(&self, height: f32) {
        self.imp().data.borrow().height.set(height);
    }
}

#[derive(Debug, Default, Clone)]
pub struct WaveData {
    pub name: String,
    pub path: Vec<String>,
    pub data: Vec<WaveChangePoint>,
    pub y_position: Cell<f32>,
    pub height: Cell<f32>,
}
