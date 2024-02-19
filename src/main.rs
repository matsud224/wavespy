mod signal_finder;
mod util;
mod wave_viewer;
use crate::signal_finder::SignalFinder;
use crate::wave_viewer::WaveViewer;
use gtk::prelude::*;
use std::fs::File;
use std::io::BufReader;
use std::rc::Rc;
use vcd::*;

fn main() {
    let application =
        gtk::Application::new(Some("com.github.matsud224.wavespy"), Default::default());
    application.connect_activate(build_ui);
    application.run();
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title(Some("WaveSpy"));
    window.set_default_size(1200, 600);
    window.present();

    let mut reader = Parser::new(BufReader::new(
        File::open("alu.vcd").expect("open file failed"),
    ));
    let header = reader.parse_header().expect("parse header failed");
    let wave_viewer = Rc::new(WaveViewer::new());
    let signal_finder = SignalFinder::new(header.items, wave_viewer.clone());

    let root_pane = gtk::Paned::builder()
        .orientation(gtk::Orientation::Horizontal)
        .start_child(&signal_finder.pane)
        .end_child(&wave_viewer.pane)
        .wide_handle(true)
        .position(250)
        .build();

    window.set_child(Some(&root_pane));

    window.show();
}
