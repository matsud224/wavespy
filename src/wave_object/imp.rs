use std::cell::RefCell;
use std::rc::Rc;

use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use super::WaveData;

#[derive(Properties, Default)]
#[properties(wrapper_type = super::WaveObject)]
pub struct WaveObject {
    #[property(name = "name", get, set, type=String,member=name)]
    pub data: Rc<RefCell<WaveData>>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for WaveObject {
    const NAME: &'static str = "WaveSpyWaveObject";
    type Type = super::WaveObject;
}

#[glib::derived_properties]
impl ObjectImpl for WaveObject {}
