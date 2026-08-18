use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserModule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub enabled: bool,
    pub stats: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcTabInfo {
    pub id: u32,
    pub url: String,
    pub title: String,
    pub is_loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcBrowserState {
    pub tabs: Vec<IpcTabInfo>,
    pub active_tab_id: Option<u32>,
    pub bookmarks: Vec<Bookmark>,
    pub modules: Vec<BrowserModule>,
    pub zoom: f64,
    pub search_engine: String,
    pub is_maximized: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum IpcIncoming {
    UiReady,
    NewTab {
        url: Option<String>,
    },
    CloseTab {
        tab_id: u32,
    },
    SwitchTab {
        tab_id: u32,
    },
    Navigate {
        url: String,
    },
    GoBack,
    GoForward,
    Reload,
    GoHome,
    SetZoom {
        zoom: f64,
    },
    ToggleBookmark {
        title: String,
        url: String,
    },
    RemoveBookmark {
        url: String,
    },
    ShowBookmarkContextMenu {
        url: String,
    },
    ToggleModule {
        module_id: String,
        enabled: bool,
    },
    TabStateUpdate {
        tab_id: Option<u32>,
        url: String,
        title: String,
        can_go_back: Option<bool>,
        can_go_forward: Option<bool>,
    },
    DragWindow,
    MinimizeWindow,
    ToggleMaximizeWindow,
    CloseWindow,
}
