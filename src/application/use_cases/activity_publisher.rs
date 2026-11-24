use crate::{
    application::ports::repositories::ActivityRepository,
    domain::entities::{Activity, ActivityKind},
};

pub async fn publish_activities(
    activity_repository: &dyn ActivityRepository,
    user_id: &str,
    kinds: Vec<ActivityKind>,
) {
    for kind in kinds {
        let _ = activity_repository
            .save(&Activity::new(user_id, kind))
            .await;
    }
}

pub fn level_up(level_before: u32, level_after: u32) -> Option<ActivityKind> {
    (level_after > level_before).then_some(ActivityKind::LevelUp { level: level_after })
}
