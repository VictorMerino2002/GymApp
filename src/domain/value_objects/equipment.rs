use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Equipment {
    BodyWeight,
    Dumbbell,
    Barbell,
    EzBar,
    TrapBar,
    Kettlebell,
    WeightPlate,
    Cable,
    Machine,
    SmithMachine,
    Sled,
    CardioMachine,
    Band,
    SuspensionTrainer,
    StabilityBall,
    MedicineBall,
    BosuBall,
    FoamRoller,
    MassageBall,
    AbWheel,
    Rope,
    Landmine,
    Sandbag,
    Macebell,
    Tire,
    Sledgehammer,
    Bench,
    PullUpBar,
}
