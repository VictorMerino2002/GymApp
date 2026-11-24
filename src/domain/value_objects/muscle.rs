use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Muscle {
    Chest,
    Shoulders,
    Traps,
    UpperBack,
    Lats,
    LowerBack,
    Biceps,
    Triceps,
    Forearms,
    Abs,
    Obliques,
    Glutes,
    Quadriceps,
    Hamstrings,
    Adductors,
    Abductors,
    Calves,
}
