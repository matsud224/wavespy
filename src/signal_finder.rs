use std::cell::RefCell;
use std::rc::Rc;

use gtk::glib;
use gtk::prelude::*;
use vcd::*;

use crate::wave_viewer;

pub struct SignalFinder {
    pub pane: gtk::Paned,
}

impl SignalFinder {
    pub fn new(hierarchy: Vec<ScopeItem>, wview: Rc<wave_viewer::WaveViewer>) -> SignalFinder {
        let scope_view = gtk::TreeView::builder().headers_visible(false).build();
        let scope_store = gtk::TreeStore::new(&[glib::Type::STRING, glib::Type::STRING]);
        create_scope_model(&scope_store, None, &hierarchy);
        let mut scope_columns: Vec<gtk::TreeViewColumn> = Vec::new();
        append_column("type", &mut scope_columns, &scope_view, None);
        append_column("name", &mut scope_columns, &scope_view, None);
        scope_view.set_model(Some(&scope_store));
        scope_view.set_activate_on_single_click(true);
        scope_view.expand_all();
        scope_view.set_enable_tree_lines(true);

        let var_view = gtk::TreeView::builder().headers_visible(true).build();
        let mut var_columns: Vec<gtk::TreeViewColumn> = Vec::new();
        append_column("type", &mut var_columns, &var_view, None);
        append_column("name", &mut var_columns, &var_view, None);

        let pane = gtk::Paned::builder()
            .orientation(gtk::Orientation::Vertical)
            .start_child(&gtk::ScrolledWindow::builder().child(&scope_view).build())
            .end_child(&gtk::ScrolledWindow::builder().child(&var_view).build())
            .build();

        let selected_hier: Rc<RefCell<Option<Vec<String>>>> = Rc::new(RefCell::new(None));

        var_view.connect_row_activated(
            glib::clone!(@strong selected_hier, @strong wview => move |var_view, path, _| {
                if let Some(m) = var_view.model() {
                    if let Some(it) = m.iter(path) {
                        if let Some(hier) = selected_hier.borrow().as_ref() {
                            let var_name = m.get_value(&it, 1).get::<String>().unwrap();
                            let var_full_path = match hier[..] {
                                [] => var_name,
                                _ => format!("{}.{}", hier.join("."), var_name)
                            };
                            wview.add_wave_by_name(&var_full_path);
                        }
                    }
                }
            }),
        );

        scope_view.connect_row_activated(glib::clone!(@strong selected_hier => move |_, path, _| {
            let var_store = gtk::TreeStore::new(&[glib::Type::STRING, glib::Type::STRING]);
            let (items, hier_strs) = get_vars(&hierarchy, path.indices().as_slice());
            create_var_model(
                &var_store,
                None,
                &items,
            );
            selected_hier.borrow_mut().replace(hier_strs);
            var_view.set_model(Some(&var_store));
        }));

        SignalFinder { pane }
    }
}

fn append_column(
    title: &str,
    v: &mut Vec<gtk::TreeViewColumn>,
    left_tree: &gtk::TreeView,
    max_width: Option<i32>,
) {
    let id = v.len() as i32;
    let renderer = gtk::CellRendererText::new();

    let column = gtk::TreeViewColumn::builder()
        .title(title)
        .resizable(true)
        .min_width(10)
        .clickable(true)
        .sort_column_id(id)
        .build();

    if let Some(max_width) = max_width {
        column.set_max_width(max_width);
        column.set_expand(true);
    }
    column.pack_start(&renderer, true);
    column.add_attribute(&renderer, "text", id);
    left_tree.append_column(&column);
    v.push(column);
}

fn create_scope_model(
    tree_store: &gtk::TreeStore,
    parent: Option<&gtk::TreeIter>,
    items: &Vec<ScopeItem>,
) {
    match parent {
        None => {
            let child = tree_store.append(parent);
            tree_store.set(&child, &[(0, &"ROOT".to_string()), (1, &"".to_string())]);
            create_scope_model(tree_store, Some(&child), items);
        }
        _ => {
            for item in items {
                if let ScopeItem::Scope(Scope {
                    scope_type,
                    identifier,
                    items,
                    ..
                }) = item
                {
                    let child = tree_store.append(parent);
                    tree_store.set(&child, &[(0, &format!("{}", scope_type)), (1, identifier)]);
                    create_scope_model(tree_store, Some(&child), items);
                }
            }
        }
    }
}

fn get_vars(items: &[ScopeItem], indices: &[i32]) -> (Vec<ScopeItem>, Vec<String>) {
    let mut current_scope = Vec::from(items);
    let mut hier_str_vec: Vec<String> = vec![];
    let (_, rest) = indices.split_first().unwrap();
    for idx in rest {
        if let ScopeItem::Scope(Scope {
            items, identifier, ..
        }) = current_scope
            .into_iter()
            .filter(|item| matches!(item, ScopeItem::Scope(_)))
            .nth(*idx as usize)
            .unwrap()
        {
            current_scope = items;
            hier_str_vec.push(identifier);
        } else {
            panic!("ScopeItem Changed");
        }
    }
    (current_scope, hier_str_vec)
}

fn create_var_model(
    tree_store: &gtk::TreeStore,
    parent: Option<&gtk::TreeIter>,
    items: &Vec<ScopeItem>,
) {
    for item in items {
        if let ScopeItem::Var(Var {
            var_type,
            size: _,
            code: _,
            reference,
            index: _,
            ..
        }) = item
        {
            let child = tree_store.append(parent);
            tree_store.set(&child, &[(0, &format!("{}", var_type)), (1, reference)]);
        }
    }
}
