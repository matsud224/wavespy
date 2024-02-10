mod signal_finder;
mod util;

use gtk::prelude::*;
use signal_finder::SignalFinder;
use std::cmp;
use std::io::BufRead;
use std::{fs::File, io::BufReader, io::Error};
use vcd::*;

type SimTime = u64;

struct WaveValue {
    pub time: SimTime,
    pub value: String,
}

impl WaveValue {
    fn new(time: SimTime, value: String) -> WaveValue {
        WaveValue { time, value }
    }
}

fn get_wave<T: BufRead>(id: &IdCode, parser: &mut Parser<T>) -> Result<Vec<WaveValue>, Error> {
    let mut current_time: SimTime = 0;
    let mut wave: Vec<WaveValue> = vec![];
    while let Some(cmd) = parser.next().transpose()? {
        match cmd {
            Command::Timestamp(t) => {
                current_time = t;
            }
            Command::ChangeScalar(i, v) if i == *id => {
                wave.push(WaveValue::new(current_time, v.to_string()));
            }
            Command::ChangeVector(i, v) if i == *id => {
                wave.push(WaveValue::new(current_time, v.to_string()));
            }
            Command::ChangeReal(i, v) if i == *id => {
                wave.push(WaveValue::new(current_time, v.to_string()));
            }
            Command::ChangeString(i, v) if i == *id => {
                wave.push(WaveValue::new(current_time, v.to_string()));
            }
            _ => (),
        }
    }
    Ok(wave)
}

fn parse_vcd(filename: &str) -> Result<(vcd::Header, Vec<WaveValue>, Vec<WaveValue>), Error> {
    let mut reader = Parser::new(BufReader::new(File::open(filename)?));
    let header = reader.parse_header()?;

    let var = &header.find_var(&["instance", "cin"]).unwrap();
    let var2 = &header.find_var(&["instance", "cout"]).unwrap();
    let wave = get_wave(&var.code, &mut reader).expect("failed to get data");
    let mut reader2 = Parser::new(BufReader::new(File::open(filename)?));
    let wave2 = get_wave(&var2.code, &mut reader2).expect("failed to get data");
    for w in &wave {
        println!("{}: {}", w.time, w.value);
    }
    Ok((header, wave, wave2))
}

fn main() {
    let application =
        gtk::Application::new(Some("com.github.matsud224.wavespy"), Default::default());
    application.connect_activate(build_ui);
    application.run();
}

fn draw_wave(cr: &gtk::cairo::Context, width: i32, wave: &[WaveValue], line_number: u64) {
    const MARGIN_BETWEEN_LINE: u64 = 5;
    const MARGIN_LEFT: u64 = 5;
    const MARGIN_TOP: u64 = 5;
    const LINE_HEIGHT: u64 = 20;

    let start_time: u64 = 0;
    let end_time: u64 = 600000;

    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.set_line_join(gtk::cairo::LineJoin::Bevel);

    let window_iter = wave.windows(2);
    for w in window_iter {
        if let [a, b] = w {
            if !(a.time > end_time || b.time < start_time) {
                let section_start_time = cmp::max(start_time, a.time);
                let section_end_time = cmp::min(end_time, b.time);

                let section_left = MARGIN_LEFT
                    + ((section_start_time - start_time) * (width as u64)
                        / (end_time - start_time + 1));
                let section_right = section_left
                    + ((section_end_time - section_start_time) * (width as u64)
                        / (end_time - start_time + 1));
                let section_top = MARGIN_TOP + line_number * (LINE_HEIGHT + MARGIN_BETWEEN_LINE);
                let section_bottom = section_top + LINE_HEIGHT;

                let section_value = if a.value == "0" { 0 } else { 1 };
                let is_value_changed = a.value != b.value;

                cr.line_to(
                    section_left as f64,
                    if section_value == 0 {
                        section_bottom as f64
                    } else {
                        section_top as f64
                    },
                );
                cr.line_to(
                    section_right as f64,
                    if section_value == 0 {
                        section_bottom as f64
                    } else {
                        section_top as f64
                    },
                );
                if is_value_changed {
                    cr.line_to(
                        section_right as f64,
                        if section_value == 0 {
                            section_top as f64
                        } else {
                            section_bottom as f64
                        },
                    );
                }
            }
        }
    }
    cr.stroke().unwrap();
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title(Some("WaveSpy"));
    window.set_default_size(1200, 600);

    let (header, wave, wave2) = parse_vcd("alu.vcd").expect("Error");

    let drawing_area = gtk::DrawingArea::builder()
        .content_width(1000)
        .content_height(500)
        .build();

    drawing_area.set_draw_func(move |_area, cr, width, _height| {
        draw_wave(cr, width, &wave, 0);
        draw_wave(cr, width, &wave2, 1);
        draw_wave(cr, width, &wave, 2);
    });

    let listbox = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .build();
    listbox.append(&gtk::Label::new(Some("foo")));
    listbox.append(&gtk::Label::new(Some("bar")));
    listbox.append(&gtk::Label::new(Some("baz")));

    let listbox2 = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .build();
    listbox2.append(&gtk::Label::new(Some("foo")));
    listbox2.append(&gtk::Label::new(Some("bar")));
    listbox2.append(&gtk::Label::new(Some("baz")));

    let signal_finder = SignalFinder::new(header.items);

    let wave_pane = gtk::Paned::builder()
        .start_child(&gtk::ScrolledWindow::builder().child(&listbox).build())
        .end_child(&gtk::ScrolledWindow::builder().child(&listbox2).build())
        .build();

    let root_pane = gtk::Paned::builder()
        .start_child(&signal_finder.pane)
        .end_child(&wave_pane)
        .build();

    window.set_child(Some(&root_pane));

    window.show();
}
