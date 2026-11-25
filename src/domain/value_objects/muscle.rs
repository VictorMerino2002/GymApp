use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Muscle {
    Chest,
    #[serde(alias = "shoulders")]
    FrontDelts,
    SideDelts,
    RearDelts,
    RotatorCuff,
    Traps,
    Neck,
    UpperBack,
    Lats,
    LowerBack,
    Biceps,
    Triceps,
    Forearms,
    Abs,
    Obliques,
    Serratus,
    HipFlexors,
    Glutes,
    Quadriceps,
    Hamstrings,
    Adductors,
    Abductors,
    Calves,
    Tibialis,
}
