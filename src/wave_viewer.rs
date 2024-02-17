use crate::wave_object::WaveObject;
use gtk::glib;
use gtk::prelude::*;
use std::cell::RefCell;
use std::cmp;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Error;
use std::rc::Rc;
use vcd::*;

type SimTime = u64;

#[derive(Clone, Debug, Default)]
pub struct WaveValue {
    pub time: SimTime,
    pub value: String,
}

impl WaveValue {
    fn new(time: SimTime, value: String) -> WaveValue {
        WaveValue { time, value }
    }
}

pub struct WaveViewer {
    pub pane: gtk::Box,
}

static ROW_HEIGHT: u64 = 25;
static MARGIN_UP_DOWN: u64 = 5;
static MARGIN_LEFT: u64 = 5;

impl WaveViewer {
    pub fn new(parent: &impl IsA<gtk::Window>) -> WaveViewer {
        let name_area = gtk::DrawingArea::builder()
            .content_width(200)
            .content_height(1000)
            .build();

        let wave_area = gtk::DrawingArea::builder()
            .content_width(1000)
            .content_height(1000)
            .build();

        let rootobjs = Rc::new(RefCell::new(vec![
            extract_wave_from_vcd("alu.vcd", vec!["instance".into(), "cin".into()]).unwrap(),
            extract_wave_from_vcd("alu.vcd", vec!["instance".into(), "cout".into()]).unwrap(),
            extract_wave_from_vcd("alu.vcd", vec!["instance".into(), "n".into()]).unwrap(),
        ]));

        name_area.set_draw_func(
            glib::clone!(@strong rootobjs => move |_area, cr, width, _height| {
                cr.set_source_rgb(0.0, 0.0, 0.0);
                cr.paint().unwrap();

                let mut y = 0;
                for wobj in rootobjs.borrow().iter() {
                    y += draw_wave_name(cr, y, width, wobj);
                }
            }),
        );

        wave_area.set_draw_func(
            glib::clone!(@strong rootobjs => move |_area, cr, width, _height| {
                cr.set_source_rgb(0.0, 0.0, 0.0);
                cr.paint().unwrap();

                let mut y = 0;
                for wobj in rootobjs.borrow().iter() {
                    y += draw_wave(cr, y, width, wobj);
                }
            }),
        );

        let hbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .build();
        let entry = gtk::Entry::builder()
            .placeholder_text("signal name")
            .build();
        let button = gtk::Button::builder().label("Add signal").build();
        button.connect_clicked(
            glib::clone!(@strong entry, @strong rootobjs, @strong name_area, @strong wave_area, @strong parent => move |_| {
                let v :Vec<String> = entry.text().split('.').map(String::from).collect();
                let wobj = extract_wave_from_vcd("alu.vcd", v);
                if let Ok(wobj) = wobj {
                    rootobjs.borrow_mut().push(wobj);
                    name_area.queue_draw();
                    wave_area.queue_draw();
                }
            }),
        );
        hbox.append(&entry);
        hbox.append(&button);

        let main_area = gtk::ScrolledWindow::builder()
            .child(
                &gtk::Paned::builder()
                    .orientation(gtk::Orientation::Horizontal)
                    .start_child(
                        &gtk::ScrolledWindow::builder()
                            .child(&name_area)
                            .vscrollbar_policy(gtk::PolicyType::Never)
                            .hscrollbar_policy(gtk::PolicyType::Automatic)
                            .build(),
                    )
                    .end_child(
                        &gtk::ScrolledWindow::builder()
                            .child(&wave_area)
                            .vscrollbar_policy(gtk::PolicyType::Never)
                            .hscrollbar_policy(gtk::PolicyType::Always)
                            .build(),
                    )
                    .wide_handle(true)
                    .build(),
            )
            .min_content_height(400)
            .vscrollbar_policy(gtk::PolicyType::Always)
            .build();

        let vbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .homogeneous(false)
            .build();
        vbox.append(&hbox);
        vbox.append(&main_area);

        WaveViewer { pane: vbox }
    }
}

fn draw_wave_name(cr: &gtk::cairo::Context, y: u64, width: i32, wobj: &WaveObject) -> u64 {
    let wdata = wobj.wave_data();

    cr.set_source_rgb(1.0, 1.0, 1.0);
    cr.set_line_join(gtk::cairo::LineJoin::Bevel);
    cr.move_to(MARGIN_LEFT as f64, (y + ROW_HEIGHT - MARGIN_UP_DOWN) as f64);
    cr.show_text(wdata.borrow().name.as_str()).unwrap();
    cr.stroke().unwrap();

    cr.move_to(0 as f64, (y + ROW_HEIGHT) as f64);
    cr.line_to(width as f64, (y + ROW_HEIGHT) as f64);
    cr.stroke().unwrap();

    ROW_HEIGHT
}

fn draw_wave(cr: &gtk::cairo::Context, y: u64, width: i32, wobj: &WaveObject) -> u64 {
    let wdata = wobj.wave_data();
    let wave = &wdata.borrow().data;
    let start_time: u64 = 0;
    let end_time: u64 = 600000;

    cr.set_source_rgb(1.0, 1.0, 1.0);
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
                let section_top = y + MARGIN_UP_DOWN;
                let section_bottom = y + ROW_HEIGHT - MARGIN_UP_DOWN;

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
                cr.stroke().unwrap();
            }
        }
    }

    cr.move_to(0 as f64, (y + ROW_HEIGHT) as f64);
    cr.line_to(width as f64, (y + ROW_HEIGHT) as f64);
    cr.stroke().unwrap();

    ROW_HEIGHT
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

fn extract_wave_from_vcd(filename: &str, path: Vec<String>) -> Result<WaveObject, Error> {
    let mut reader = Parser::new(BufReader::new(File::open(filename)?));
    let header = reader.parse_header()?;
    let var = &header.find_var(&path).unwrap();
    let wave = get_wave(&var.code, &mut reader).expect("failed to get data");
    Ok(WaveObject::new(path.join("."), path, wave))
}
