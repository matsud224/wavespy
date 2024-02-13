use crate::wave_object::WaveObject;
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use std::cmp;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Error;
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
    pub pane: gtk::Paned,
}

impl WaveViewer {
    pub fn new(parent: &impl IsA<gtk::Window>) -> WaveViewer {
        let drawing_area = gtk::DrawingArea::builder()
            .content_width(1000)
            .content_height(500)
            .build();

        let rootobjs = vec![
            extract_wave_from_vcd("alu.vcd", vec!["instance".into(), "cin".into()]).unwrap(),
            extract_wave_from_vcd("alu.vcd", vec!["instance".into(), "cout".into()]).unwrap(),
            extract_wave_from_vcd("alu.vcd", vec!["instance".into(), "n".into()]).unwrap(),
        ];
        let root = create_root_model(&rootobjs);
        let model = gtk::TreeListModel::new(
            root.clone().upcast::<gio::ListModel>(),
            false,
            true,
            create_model,
        );

        let selection_model = gtk::SingleSelection::new(Some(model));
        let factory = gtk::SignalListItemFactory::new();
        let list_view = gtk::ListView::new(
            Some(selection_model),
            None as Option<gtk::SignalListItemFactory>,
        );
        let list_view_cloned = list_view.clone();
        factory.connect_setup(move |_, list_item| {
            let expander = gtk::TreeExpander::new();
            let label = gtk::Label::new(None);
            expander.set_child(Some(&label));
            expander.set_margin_top(5);
            expander.set_margin_bottom(5);
            list_item.set_child(Some(&expander));
        });
        factory.connect_bind(glib::clone!(@strong drawing_area => move |_, list_item| {
            let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
            if let Some(row) = list_item.item().and_downcast::<gtk::TreeListRow>() {
                if let Some(wobj) = row.item().and_downcast::<WaveObject>() {
                    if let Some(expander) = list_item.child().and_downcast::<gtk::TreeExpander>() {
                        expander.set_list_row(Some(&row));
                        if let Some(label) = expander.child().and_downcast::<gtk::Label>() {
                            label.set_label(&wobj.name());
                            let bounds = expander.compute_bounds(&list_view_cloned);
                            if let Some(bounds)  = bounds {
                                wobj.set_y_position(bounds.y());
                                wobj.set_height(bounds.height());
                                drawing_area.queue_draw();
                            }
                        }
                    }
                }
            }
        }));

        list_view.set_factory(Some(&factory));

        drawing_area.set_draw_func(
            glib::clone!(@strong root => move |_area, cr, width, _height| {
                for i in 0..root.n_items() {
                    draw_wave(cr, width, &root.item(i).and_downcast::<WaveObject>().unwrap());
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
            glib::clone!(@strong entry, @strong root, @strong drawing_area, @strong parent => move |_| {
                let v :Vec<String> = entry.text().split('.').map(String::from).collect();
                let wobj = &extract_wave_from_vcd("alu.vcd", v);
                if let Ok(wobj) = wobj {
                    root.append(wobj);
                    drawing_area.queue_draw();
                }
            }),
        );
        hbox.append(&entry);
        hbox.append(&button);

        let vbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .build();
        vbox.append(&list_view);
        vbox.append(&hbox);

        let pane = gtk::Paned::builder()
            .orientation(gtk::Orientation::Horizontal)
            .start_child(&gtk::ScrolledWindow::builder().child(&vbox).build())
            .end_child(&gtk::ScrolledWindow::builder().child(&drawing_area).build())
            .build();

        WaveViewer { pane }
    }
}

fn create_root_model(wobjs: &[WaveObject]) -> gio::ListStore {
    let result = gio::ListStore::new::<WaveObject>();
    for wobj in wobjs {
        result.append(wobj);
    }
    result
}

fn create_model(_obj: &gtk::glib::Object) -> Option<gio::ListModel> {
    /*
    if let Some(iobj) = obj.downcast_ref::<WaveObject>() {
        if iobj.children().is_empty() {
            None
        } else {
            let result = gio::ListStore::new::<WaveObject>();
            for i in iobj.children() {
                result.append(&WaveObject::new(i, &[]));
            }
            Some(result.upcast::<gio::ListModel>())
        }
    } else {
        None
    }
    */
    None
}

fn draw_wave(cr: &gtk::cairo::Context, width: i32, wobj: &WaveObject) {
    const MARGIN_LEFT: u64 = 5;

    let wdata = wobj.wave_data();
    let wave = &wdata.borrow().data;
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
                let section_top = wdata.borrow().y_position.get();
                let section_bottom = wdata.borrow().y_position.get() + wdata.borrow().height.get();

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
