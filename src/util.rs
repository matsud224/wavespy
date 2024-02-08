use vcd::*;

#[allow(dead_code)]
pub fn print_items(items: &Vec<ScopeItem>, depth: usize) {
    for item in items {
        match item {
            ScopeItem::Scope(Scope {
                scope_type,
                identifier,
                items,
                ..
            }) => {
                println!("{}{} {}", "-".repeat(depth), scope_type, identifier);
                print_items(items, depth + 1);
            }
            ScopeItem::Var(Var {
                var_type,
                size: _,
                code: _,
                reference,
                index: _,
                ..
            }) => {
                println!("{}{} {}", "-".repeat(depth), var_type, reference);
            }
            _ => (),
        }
    }
}
