use std::{collections::HashMap, sync::Mutex};

use async_trait::async_trait;

use crate::{
    application::ports::repositories::RoutineRepository,
    domain::{
        entities::Routine,
        value_objects::{Error, Page},
    },
};

#[derive(Default)]
pub struct InMemoryRoutineRepository {
    routines: Mutex<HashMap<String, Routine>>,
}

impl InMemoryRoutineRepository {
    pub fn get(&self, id: &str) -> Option<Routine> {
        self.routines.lock().unwrap().get(id).cloned()
    }
}

#[async_trait]
impl RoutineRepository for InMemoryRoutineRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Routine>, Error> {
        Ok(self.get(id))
    }

    async fn find_by_ids(&self, ids: &[String]) -> Result<Vec<Routine>, Error> {
        let routines = self.routines.lock().unwrap();
        Ok(ids
            .iter()
            .filter_map(|id| routines.get(id).cloned())
            .collect())
    }

    async fn find_paginated(
        &self,
        _limit: u16,
        _cursor: Option<&str>,
    ) -> Result<Page<Routine>, Error> {
        unimplemented!()
    }

    async fn find_by_author(
        &self,
        _author_id: &str,
        _limit: u16,
        _cursor: Option<&str>,
    ) -> Result<Page<Routine>, Error> {
        unimplemented!()
    }

    async fn save(&self, routine: &Routine) -> Result<(), Error> {
        self.routines
            .lock()
            .unwrap()
            .insert(routine.id.clone(), routine.clone());
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), Error> {
        self.routines.lock().unwrap().remove(id);
        Ok(())
    }
}
