use super::super::app::flatten_tree_items;
use crate::runtime::{TreeItemNode, TreeRowView};

#[test]
fn flatten_tree_items_includes_only_expanded_children() {
    let expanded_parent = TreeItemNode::new("Parent").child(TreeItemNode::new("Child"));
    let collapsed_parent = TreeItemNode::new("Collapsed")
        .children(vec![TreeItemNode::new("Hidden")])
        .expanded(false);

    let rows = flatten_tree_items(vec![expanded_parent, collapsed_parent]);

    assert_eq!(rows.len(), 3);
    assert_row(&rows[0], "Parent", 0, true, true);
    assert_row(&rows[1], "Child", 1, false, false);
    assert_row(&rows[2], "Collapsed", 0, true, false);
}

fn assert_row(row: &TreeRowView, label: &str, depth: usize, has_children: bool, expanded: bool) {
    assert_eq!(row.label, label);
    assert_eq!(row.depth, depth);
    assert_eq!(row.has_children, has_children);
    assert_eq!(row.expanded, expanded);
}
