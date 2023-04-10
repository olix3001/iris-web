use std::{collections::HashMap, sync::Arc, any::{Any, TypeId}};

/// Internal struct that holds data
#[derive(Debug, Clone)]
pub struct DataContainer {
    data: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl Default for DataContainer {
    fn default() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl DataContainer {
    /// Adds data to the container
    pub fn add<T: Any + Send + Sync>(&mut self, value: T) {
        self.data.insert(TypeId::of::<T>(), Arc::new(Arc::new(value)));
    }

    /// Adds data with an existing `Arc` to the container
    pub fn add_arc<T: Any + Send + Sync>(&mut self, value: Arc<T>) {
        self.data.insert(TypeId::of::<T>(), Arc::new(value));
    } 

    /// Gets data from the container
    pub fn get<T: Any + Send + Sync>(&self) -> Option<Arc<T>> {
        self.data.get(&TypeId::of::<T>()).and_then(|data| data.downcast_ref::<Arc<T>>()).cloned()
    }

    /// Combines data from two containers creating a new one
    pub fn combine(&self, other: &Self) -> Self {
        let mut data = self.data.clone();

        for (key, value) in &other.data {
            data.insert(*key, value.clone());
        }

        Self {
            data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_container() {
        let mut container = DataContainer::default();

        container.add(5i32);

        assert_eq!(container.get::<i32>(), Some(Arc::new(5)));
    }
}