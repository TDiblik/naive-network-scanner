use super::workspace_models::TabsContext;

#[derive(Clone, PartialEq)]
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

pub fn default_tabs() -> TabsContext {
    let upper_tabs = vec![
        WorkspaceTab::new("meta_tab", "General"),
        WorkspaceTab::new("discovery_shared_tab", "Discovery"),
        WorkspaceTab::new("discovery_outside_tab", "Discovery (outside network)"),
        WorkspaceTab::new("discovery_inside_tab", "Discovery (inside network)"),
    ];
    let middle_left_tabs = vec![
        WorkspaceTab::new("topology_overview_tab", "Network Topology"),
        WorkspaceTab::new("cnc_overview_tab", "CnC Server"),
    ];
    let middle_right_tabs = vec![
        WorkspaceTab::new("notes_tab", "Notes"),
        WorkspaceTab::new("status_tab", "Status Info"),
        WorkspaceTab::new("performed_steps_tab", "Performed steps"),
    ];
    let bottom_tabs = vec![
        WorkspaceTab::new("hints_tab", "Hints"),
        WorkspaceTab::new("terminal_tab", "Terminal (1)"),
    ];
    let all_tabs = [
        upper_tabs.clone(),
        middle_left_tabs.clone(),
        middle_right_tabs.clone(),
        bottom_tabs.clone(),
    ]
    .concat();

    let mut tab_tree = egui_dock::Tree::new(upper_tabs);
    let [_, tabs_below] = tab_tree.split_below(egui_dock::NodeIndex::root(), 0.1, middle_left_tabs);
    let [tabs_left, _] = tab_tree.split_right(tabs_below, 0.75, middle_right_tabs);
    tab_tree.split_below(tabs_left, 0.85, bottom_tabs);

    TabsContext {
        tab_tree,
        default_tabs: all_tabs,
    }
}
