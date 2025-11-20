use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::timeout;

use super::super::app::flatten_tree_items;
use super::super::dispatcher::AppMessage;
use crate::runtime::{App, Element, RuntimeDriver, TreeItemNode, TreeRowView, component};

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

#[tokio::test]
async fn app_run_uses_custom_runtime_driver() {
    let driver = TestRuntimeDriver::default();
    let app = App::new("DriverTest", component("Unit", |_ctx| Element::Empty))
        .with_driver(driver.clone());

    timeout(Duration::from_millis(200), app.run())
        .await
        .expect("runtime exited")
        .expect("app run succeeds");

    let (terminal, tick, shutdown) = driver.call_counts();
    assert_eq!(terminal, 1);
    assert_eq!(tick, 1);
    assert_eq!(shutdown, 1);
}

#[derive(Clone, Default)]
struct TestRuntimeDriver {
    inner: Arc<TestRuntimeDriverInner>,
}

struct TestRuntimeDriverInner {
    terminal_calls: AtomicUsize,
    tick_calls: AtomicUsize,
    shutdown_calls: AtomicUsize,
}

impl Default for TestRuntimeDriverInner {
    fn default() -> Self {
        Self {
            terminal_calls: AtomicUsize::new(0),
            tick_calls: AtomicUsize::new(0),
            shutdown_calls: AtomicUsize::new(0),
        }
    }
}

impl RuntimeDriver for TestRuntimeDriver {
    fn spawn_terminal_events(&self, tx: mpsc::Sender<AppMessage>) -> JoinHandle<()> {
        self.inner.terminal_calls.fetch_add(1, Ordering::SeqCst);
        tokio::spawn(async move {
            let _ = tx.send(AppMessage::Shutdown).await;
        })
    }

    fn spawn_tick_loop(&self, _tx: mpsc::Sender<AppMessage>, _rate: Duration) -> JoinHandle<()> {
        self.inner.tick_calls.fetch_add(1, Ordering::SeqCst);
        tokio::spawn(async {})
    }

    fn spawn_shutdown_watcher(&self, _tx: mpsc::Sender<AppMessage>) -> JoinHandle<()> {
        self.inner.shutdown_calls.fetch_add(1, Ordering::SeqCst);
        tokio::spawn(async {})
    }
}

impl TestRuntimeDriver {
    fn call_counts(&self) -> (usize, usize, usize) {
        (
            self.inner.terminal_calls.load(Ordering::SeqCst),
            self.inner.tick_calls.load(Ordering::SeqCst),
            self.inner.shutdown_calls.load(Ordering::SeqCst),
        )
    }
}
