use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DockPlusPlacement {
    Center,
    LeftEdge,
    RightEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DockAction {
    Sort,
    Deadline,
    HideDone,
    DeleteDone,
    BatchDelete,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DockConfig {
    pub plus_placement: DockPlusPlacement,
    pub actions: Vec<DockAction>,
}

impl Default for DockConfig {
    fn default() -> Self {
        Self {
            plus_placement: DockPlusPlacement::Center,
            actions: vec![DockAction::Sort, DockAction::Deadline],
        }
    }
}

impl DockConfig {
    pub fn normalized(mut self) -> Self {
        let mut unique = Vec::new();
        for action in self.actions {
            if !unique.contains(&action) && unique.len() < 4 {
                unique.push(action);
            }
        }
        self.actions = unique;
        self
    }
}
