pub struct WorkspaceTab {
    pub id: String,
    pub title: String,
}

impl WorkspaceTab {
    pub fn new(id: &str, title: &str) -> WorkspaceTab {
        Self {
            id: id.to_owned(),
            title: title.to_owned(),
        }
    }
}

pub fn default_tab_tree() -> egui_dock::Tree<WorkspaceTab> {
    let mut tab_tree = egui_dock::Tree::new(vec![
        WorkspaceTab::new("meta_tab", "General"),
        WorkspaceTab::new("discovery_shared_tab", "Discovery"),
        WorkspaceTab::new("discovery_outside_tab", "Discovery (outside network)"),
        WorkspaceTab::new("discovery_inside_tab", "Discovery (inside network)"),
    ]);

    let [_, tabs_below] = tab_tree.split_below(
        egui_dock::NodeIndex::root(),
        0.1,
        vec![
            WorkspaceTab::new("topology_overview_tab", "Network Topology"),
            WorkspaceTab::new("cnc_overview_tab", "CnC Server"),
        ],
    );

    let [tabs_left, _] = tab_tree.split_right(
        tabs_below,
        0.75,
        vec![
            WorkspaceTab::new("notes_tab", "Notes"),
            WorkspaceTab::new("status_tab", "Status Info"),
            WorkspaceTab::new("performed_steps_tab", "Performed steps"),
        ],
    );

    tab_tree.split_below(
        tabs_left,
        0.85,
        vec![
            WorkspaceTab::new("hints_tab", "Hints"),
            WorkspaceTab::new("terminal_tab", "Terminal (1)"),
        ],
    );

    tab_tree
}
