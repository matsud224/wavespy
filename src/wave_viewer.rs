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

#[derive(Clone, PartialEq, Debug)]
pub enum WaveValue {
    Scalar(vcd::Value),
    Vector(vcd::Vector),
    Custom(String),
}

#[derive(Clone, Debug)]
pub struct WaveChangePoint {
    pub time: SimTime,
    pub value: WaveValue,
}

impl WaveChangePoint {
    fn new(time: SimTime, value: WaveValue) -> WaveChangePoint {
        WaveChangePoint { time, value }
    }
}

#[derive(Debug, Default, Clone)]
pub struct WaveData {
    pub name: String,
    pub path: Vec<String>,
    pub data: Vec<WaveChangePoint>,
}

impl WaveData {
    fn new(name: String, path: Vec<String>, data: Vec<WaveChangePoint>) -> Self {
        WaveData { name, path, data }
    }
}

pub struct WaveViewer {
    pub pane: gtk::Box,
    name_area: gtk::DrawingArea,
    value_area: gtk::DrawingArea,
    wave_area: gtk::DrawingArea,
    waves: Rc<RefCell<Vec<WaveData>>>,
}

static ROW_HEIGHT: u64 = 30;
static MARGIN_UP_DOWN: u64 = 5;
static MARGIN_SIDE: u64 = 5;

impl WaveViewer {
    pub fn new() -> WaveViewer {
        let name_area = gtk::DrawingArea::builder().build();
        let value_area = gtk::DrawingArea::builder().build();
        let wave_area = gtk::DrawingArea::builder().build();

        let waves = Rc::new(RefCell::new(vec![
            extract_wave_from_vcd("alu.vcd", vec!["instance".into(), "cin".into()]).unwrap(),
            extract_wave_from_vcd("alu.vcd", vec!["instance".into(), "cout".into()]).unwrap(),
            extract_wave_from_vcd("alu.vcd", vec!["instance".into(), "cmd[1:0]".into()]).unwrap(),
        ]));

        name_area.set_draw_func(
            glib::clone!(@strong waves => move |area, cr, width, _height| {
                draw_background(cr);

                let mut max_w : u64 = 0;
                let mut y = 0;
                for wobj in waves.borrow().iter() {
                    let (w, h) = draw_wave_name(cr, y, width, wobj);
                    y += h;
                    max_w = u64::max(max_w, w);
                }

                area.set_content_width(max_w as i32);
                area.set_content_height(y as i32);
            }),
        );

        value_area.set_draw_func(
            glib::clone!(@strong waves => move |area, cr, width, _height| {
                draw_background(cr);

                let mut max_w : u64 = 0;
                let mut y = 0;
                for wobj in waves.borrow().iter() {
                    let (w, h) = draw_wave_value(cr, y, width, wobj);
                    y += h;
                    max_w = u64::max(max_w, w);
                }

                area.set_content_width(max_w as i32);
                area.set_content_height(y as i32);
            }),
        );

        wave_area.set_draw_func(
            glib::clone!(@strong waves => move |area, cr, width, _height| {
                draw_background(cr);

                let mut y = 0;
                for wobj in waves.borrow().iter() {
                    y += draw_wave(cr, y, width, wobj);
                }

                area.set_content_height(y as i32);
            }),
        );

        let scroll_hbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .build();
        let wave_scrollbar = gtk::Scrollbar::builder()
            .adjustment(&gtk::Adjustment::new(
                10.0, 0.0, 10000000.0, 1.0, 1.0, 1000.0,
            ))
            .hexpand(true)
            .build();
        scroll_hbox.append(&gtk::Button::from_icon_name("go-first-symbolic"));
        scroll_hbox.append(&gtk::Button::from_icon_name("go-last-symbolic"));
        scroll_hbox.append(&gtk::Button::from_icon_name("go-previous-symbolic"));
        scroll_hbox.append(&gtk::Button::from_icon_name("go-next-symbolic"));
        scroll_hbox.append(&gtk::Separator::new(gtk::Orientation::Vertical));
        scroll_hbox.append(&gtk::Label::new(Some("0 ns")));
        scroll_hbox.append(&gtk::Separator::new(gtk::Orientation::Vertical));
        scroll_hbox.append(&wave_scrollbar);

        let main_area = gtk::ScrolledWindow::builder()
            .child(
                &gtk::Paned::builder()
                    .orientation(gtk::Orientation::Horizontal)
                    .start_child(
                        &gtk::Paned::builder()
                            .orientation(gtk::Orientation::Horizontal)
                            .position(100)
                            .start_child(
                                &gtk::ScrolledWindow::builder()
                                    .child(&name_area)
                                    .vscrollbar_policy(gtk::PolicyType::Never)
                                    .hscrollbar_policy(gtk::PolicyType::Automatic)
                                    .vexpand(true)
                                    .hexpand(true)
                                    .build(),
                            )
                            .end_child(
                                &gtk::ScrolledWindow::builder()
                                    .child(&value_area)
                                    .vscrollbar_policy(gtk::PolicyType::Never)
                                    .hscrollbar_policy(gtk::PolicyType::Automatic)
                                    .vexpand(true)
                                    .hexpand(true)
                                    .build(),
                            )
                            .build(),
                    )
                    .end_child(&wave_area)
                    .wide_handle(true)
                    .position(200)
                    .build(),
            )
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .build();

        let vbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .homogeneous(false)
            .build();
        vbox.append(&main_area);
        vbox.append(&scroll_hbox);

        WaveViewer {
            pane: vbox,
            name_area,
            value_area,
            wave_area,
            waves,
        }
    }

    pub fn add_wave_by_name(&self, name: &str) {
        let v: Vec<String> = name.split('.').map(String::from).collect();
        let wdata = extract_wave_from_vcd("alu.vcd", v);
        if let Ok(wdata) = wdata {
            self.waves.borrow_mut().push(wdata);
            self.redraw();
        }
    }

    fn redraw(&self) {
        self.name_area.queue_draw();
        self.value_area.queue_draw();
        self.wave_area.queue_draw();
    }
}

fn draw_background(cr: &gtk::cairo::Context) {
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.paint().unwrap();
}

enum Align {
    Right,
    Left,
}

fn draw_text(cr: &gtk::cairo::Context, y: u64, width: i32, align: Align, text: &str) {
    let text_ext = cr.text_extents(text).unwrap();
    match align {
        Align::Left => {
            cr.move_to(MARGIN_SIDE as f64, (y + ROW_HEIGHT - MARGIN_UP_DOWN) as f64);
        }
        Align::Right => {
            cr.move_to(
                width as f64 - MARGIN_SIDE as f64 - text_ext.width(),
                (y + ROW_HEIGHT - MARGIN_UP_DOWN) as f64,
            );
        }
    }

    cr.show_text(text).ok();
}

fn draw_wave_name(cr: &gtk::cairo::Context, y: u64, width: i32, wdata: &WaveData) -> (u64, u64) {
    let text = wdata.name.clone();
    let text_ext = cr.text_extents(&text).unwrap();

    cr.set_source_rgb(1.0, 1.0, 1.0);
    cr.set_line_join(gtk::cairo::LineJoin::Bevel);

    draw_text(cr, y, width, Align::Left, &text);
    cr.stroke().unwrap();

    cr.move_to(0 as f64, (y + ROW_HEIGHT) as f64);
    cr.line_to(width as f64, (y + ROW_HEIGHT) as f64);
    cr.stroke().unwrap();

    (MARGIN_SIDE * 2 + text_ext.width() as u64, ROW_HEIGHT)
}

fn draw_wave_value(cr: &gtk::cairo::Context, y: u64, width: i32, _wdata: &WaveData) -> (u64, u64) {
    let text = ((width as u64 + y) % 2).to_string().repeat(32);
    let text_ext = cr.text_extents(&text).unwrap();

    cr.set_source_rgb(1.0, 1.0, 1.0);
    cr.set_line_join(gtk::cairo::LineJoin::Bevel);

    draw_text(cr, y, width, Align::Right, &text);
    cr.stroke().unwrap();

    cr.move_to(0 as f64, (y + ROW_HEIGHT) as f64);
    cr.line_to(width as f64, (y + ROW_HEIGHT) as f64);
    cr.stroke().unwrap();

    (MARGIN_SIDE * 2 + text_ext.width() as u64, ROW_HEIGHT)
}

fn draw_wave(cr: &gtk::cairo::Context, y: u64, width: i32, wdata: &WaveData) -> u64 {
    let wave = &wdata.data;
    let start_time: u64 = 0;
    let end_time: u64 = 50000;

    cr.set_source_rgb(1.0, 1.0, 1.0);
    cr.set_line_join(gtk::cairo::LineJoin::Bevel);

    let window_iter = wave.windows(2);
    for w in window_iter {
        if let [a, b] = w {
            if !(a.time > end_time || b.time < start_time) {
                let section_start_time = cmp::max(start_time, a.time);
                let section_end_time = cmp::min(end_time, b.time);

                let section_left = (section_start_time - start_time) * (width as u64)
                    / (end_time - start_time + 1);
                let section_right = section_left
                    + ((section_end_time - section_start_time) * (width as u64)
                        / (end_time - start_time + 1));
                let section_top = y + MARGIN_UP_DOWN;
                let section_bottom = y + ROW_HEIGHT - MARGIN_UP_DOWN;

                let is_value_changed = a.value != b.value;

                let value_to_hline_y_pos = |val: &vcd::Value| -> u64 {
                    match val {
                        vcd::Value::V0 => section_top,
                        vcd::Value::V1 => section_bottom,
                        vcd::Value::X => section_top + ROW_HEIGHT / 2,
                        vcd::Value::Z => section_top + ROW_HEIGHT / 2,
                    }
                };

                match (&a.value, &b.value) {
                    (WaveValue::Scalar(v1), WaveValue::Scalar(v2)) => {
                        cr.line_to(section_left as f64, value_to_hline_y_pos(v1) as f64);
                        cr.line_to(section_right as f64, value_to_hline_y_pos(v1) as f64);
                        cr.line_to(section_right as f64, value_to_hline_y_pos(v2) as f64);
                    }
                    (WaveValue::Vector(v1), WaveValue::Vector(_v2)) => {
                        cr.line_to(section_left as f64, section_top as f64);
                        cr.line_to(section_right as f64, section_top as f64);
                        cr.stroke().unwrap();

                        cr.move_to(section_left as f64, section_bottom as f64);
                        cr.line_to(section_right as f64, section_bottom as f64);
                        cr.stroke().unwrap();

                        if is_value_changed {
                            cr.move_to(section_right as f64, section_top as f64);
                            cr.line_to(section_right as f64, section_bottom as f64);
                            cr.stroke().unwrap();
                        }

                        cr.move_to((section_left + 2) as f64, (section_bottom - 2) as f64);
                        cr.show_text(&v1.to_string()).ok();
                        cr.stroke().unwrap();
                    }
                    (WaveValue::Custom(v1), WaveValue::Custom(_v2)) => {
                        cr.line_to(section_left as f64, section_top as f64);
                        cr.line_to(section_right as f64, section_top as f64);
                        cr.stroke().unwrap();

                        cr.move_to(section_left as f64, section_bottom as f64);
                        cr.line_to(section_right as f64, section_bottom as f64);
                        cr.stroke().unwrap();

                        if is_value_changed {
                            cr.move_to(section_right as f64, section_top as f64);
                            cr.line_to(section_right as f64, section_bottom as f64);
                            cr.stroke().unwrap();
                        }

                        cr.move_to((section_left + 2) as f64, (section_bottom - 2) as f64);
                        cr.show_text(&v1.to_string()).ok();
                        cr.stroke().unwrap();
                    }
                    _ => (),
                }
            }
        }
    }

    cr.stroke().unwrap();

    cr.move_to(0 as f64, (y + ROW_HEIGHT) as f64);
    cr.line_to(width as f64, (y + ROW_HEIGHT) as f64);
    cr.stroke().unwrap();

    ROW_HEIGHT
}

fn get_wave<T: BufRead>(
    id: &IdCode,
    parser: &mut Parser<T>,
) -> Result<Vec<WaveChangePoint>, Error> {
    let mut current_time: SimTime = 0;
    let mut wave: Vec<WaveChangePoint> = vec![];
    while let Some(cmd) = parser.next().transpose()? {
        match cmd {
            Command::Timestamp(t) => {
                current_time = t;
            }
            Command::ChangeScalar(i, v) if i == *id => {
                wave.push(WaveChangePoint::new(current_time, WaveValue::Scalar(v)));
            }
            Command::ChangeVector(i, v) if i == *id => {
                wave.push(WaveChangePoint::new(current_time, WaveValue::Vector(v)));
            }
            Command::ChangeReal(i, v) if i == *id => {
                wave.push(WaveChangePoint::new(
                    current_time,
                    WaveValue::Custom(v.to_string()),
                ));
            }
            Command::ChangeString(i, v) if i == *id => {
                wave.push(WaveChangePoint::new(
                    current_time,
                    WaveValue::Custom(v.to_string()),
                ));
            }
            _ => (),
        }
    }
    Ok(wave)
}

fn extract_wave_from_vcd(filename: &str, path: Vec<String>) -> Result<WaveData, Error> {
    let mut reader = Parser::new(BufReader::new(File::open(filename)?));
    let header = reader.parse_header()?;
    let var = &header.find_var(&path).unwrap();
    let wave = get_wave(&var.code, &mut reader).expect("failed to get data");
    Ok(WaveData::new(path.join("."), path, wave))
}
