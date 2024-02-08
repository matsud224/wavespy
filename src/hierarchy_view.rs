use gtk::glib;
use gtk::prelude::*;
use vcd::*;

pub struct HierarchyView {
    pub tree_view: gtk::TreeView,
    pub tree_store: gtk::TreeStore,
    pub tree_view2: gtk::TreeView,
    pub tree_store2: gtk::TreeStore,
    pub pane: gtk::Paned,
}

impl HierarchyView {
    pub fn new(hierarchy: &Vec<ScopeItem>) -> HierarchyView {
        let tree_view = gtk::TreeView::builder().headers_visible(false).build();
        let tree_store = gtk::TreeStore::new(&[glib::Type::STRING, glib::Type::STRING]);
        create_model(&tree_store, None, hierarchy);
        let mut columns: Vec<gtk::TreeViewColumn> = Vec::new();
        append_column("type", &mut columns, &tree_view, None);
        append_column("name", &mut columns, &tree_view, None);
        tree_view.set_model(Some(&tree_store));

        let tree_view2 = gtk::TreeView::builder().headers_visible(false).build();
        let tree_store2 = gtk::TreeStore::new(&[glib::Type::STRING, glib::Type::STRING]);
        create_model(&tree_store2, None, hierarchy);
        let mut columns2: Vec<gtk::TreeViewColumn> = Vec::new();
        append_column("type", &mut columns2, &tree_view2, None);
        append_column("name", &mut columns2, &tree_view2, None);
        tree_view2.set_model(Some(&tree_store2));

        let pane = gtk::Paned::builder()
            .orientation(gtk::Orientation::Vertical)
            .start_child(&gtk::ScrolledWindow::builder().child(&tree_view).build())
            .end_child(&gtk::ScrolledWindow::builder().child(&tree_view2).build())
            .build();

        HierarchyView {
            tree_view,
            tree_store,
            tree_view2,
            tree_store2,
            pane,
        }
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

    if title != "process name" {
        renderer.set_xalign(1.0);
    }

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

fn create_model(
    tree_store: &gtk::TreeStore,
    parent: Option<&gtk::TreeIter>,
    items: &Vec<ScopeItem>,
) {
    match parent {
        None => {
            let child = tree_store.append(parent);
            tree_store.set(&child, &[(0, &"ROOT".to_string()), (1, &"".to_string())]);
            create_model(tree_store, Some(&child), items);
        }
        _ => {
            for item in items {
                match item {
                    ScopeItem::Scope(Scope {
                        scope_type,
                        identifier,
                        items,
                        ..
                    }) => {
                        let child = tree_store.append(parent);
                        tree_store.set(&child, &[(0, &format!("{}", scope_type)), (1, identifier)]);
                        create_model(tree_store, Some(&child), items);
                    }
                    ScopeItem::Var(Var {
                        var_type,
                        size: _,
                        code: _,
                        reference,
                        index: _,
                        ..
                    }) => {
                        let child = tree_store.append(parent);
                        tree_store.set(&child, &[(0, &format!("{}", var_type)), (1, reference)]);
                    }
                    _ => (),
                }
            }
        }
    }
}
