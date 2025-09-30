//! A module for managing scheduled items that are periodically polled.
//!
//! The `schedule` module provides structures and traits to manage objects
//! that need to be executed, updated, or checked at regular intervals.
//! Each item must implement the `Schedulable` trait, which defines a unique
//! identifier and an associated interval.
//!
//! The `Schedule` struct maintains:
//! - A mapping of item `id` to the items themselves for fast lookup.
//! - A mapping of `interval` to sets of item `id`, allowing efficient
//!   retrieval of all items that should be polled at a given interval.
//!
//! # Example
//!
//! ```rust
//! use std::collections::HashSet;
//!
//! use limon_core::schedule::{Schedule, Schedulable};
//!
//! struct Task {
//!     id: i64,
//!     interval: i64,
//! }
//!
//! impl Schedulable for Task {
//!     type Id = i64;
//!     type Interval = i64;
//!
//!     fn get_id(&self) -> Self::Id { self.id }
//!     fn get_interval(&self) -> Self::Interval { self.interval }
//! }
//!
//! let schedule: Schedule<Task> = Schedule::new();
//!
//! # tokio_test::block_on(async {
//! schedule.insert(Task { id: 1, interval: 30 }).await;
//! schedule.insert(Task { id: 2, interval: 60 }).await;
//!
//! assert_eq!(schedule.get_due(0, 90).await.len(), 2);
//! # })
//! ```

use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::Arc;

use tokio::sync::RwLock;

/// A trait for items that can be scheduled.
///
/// This trait defines the necessary requirements for an item to be
/// stored and managed by a [Schedule]. Each item must have a unique
/// identifier `id` and an associated `interval`. Both types must
/// support hashing and equality checks, and be convertible to `i64`.
pub trait Schedulable {
  /// The unique identifier for the item.
  type Id: Eq + Hash + Into<i64> + Copy;

  /// The interval associated with the item.
  type Interval: Eq + Hash + Into<i64> + Copy;

  /// Returns the unique identifier of the item.
  fn get_id(&self) -> Self::Id;

  /// Returns the interval of the item.
  fn get_interval(&self) -> Self::Interval;
}

/// A schedule for managing [Schedulable] items.
///
/// The [Schedule] structure stores items indexed by their unique
/// identifiers and groups item `id` by their `interval`. This allows
/// efficient lookup of items by `id` and retrieval of all `id` in a
/// given interval.
///
/// | Operation | Time complexity |
/// |-----------|-----------------|
/// | Get       | O(1)            |
/// | Get due   | O(m)            |
/// | Insert    | O(1)            |
/// | Remove    | O(1)            |
///
/// **m** - it's amount of unique intervals.
pub struct Schedule<Item: Schedulable> {
  items: RwLock<HashMap<Item::Id, Arc<Item>>>,
  intervals: RwLock<HashMap<Item::Interval, HashSet<Item::Id>>>,
}

impl<Item: Schedulable> Schedule<Item> {
  /// Create a new schedule.
  pub fn new() -> Self {
    Self {
      items: RwLock::new(HashMap::new()),
      intervals: RwLock::new(HashMap::new()),
    }
  }

  /// Get an item by `id`.
  pub async fn get(&self, id: Item::Id) -> Option<Arc<Item>> {
    self.items.read().await.get(&id).cloned()
  }

  /// Get items that are included in the interval `from` and `to`.
  ///
  /// An element is included in the interval if there is at least
  /// one value between `from` and `to` that is divisible by
  /// the item's [interval](Schedulable::Interval) without a remainder.
  ///
  /// `from` and `to` should be > 0 and `from` should be <= `to`.
  pub async fn get_due(&self, from: i64, to: i64) -> Vec<Arc<Item>> {
    let mut result = Vec::new();
    let intervals = self.intervals.read().await;

    for (interval, ids) in intervals.iter() {
      let interval = (*interval).into();
      let next_check = ((from + interval - 1) / interval) * interval;

      if next_check <= to {
        let guard = self.items.read().await;

        for id in ids {
          if let Some(item) = guard.get(id) {
            result.push(item.clone());
          }
        }
      }
    }

    result
  }

  /// Insert an item into schedule.
  ///
  /// If an item with this `id` is already in the schedule, it will be replaced.
  pub async fn insert(&self, item: Item) {
    let id = item.get_id();
    let interval = item.get_interval();

    {
      let mut intervals = self.intervals.write().await;

      if let Some(ids_set) = intervals.get_mut(&interval) {
        ids_set.insert(id);
      } else {
        let mut set = HashSet::new();
        set.insert(id);

        intervals.insert(interval, set);
      }
    }

    {
      let mut items = self.items.write().await;

      items.insert(id, Arc::new(item));
    }
  }

  /// Remove an item by `id` from the schedule if it exists.
  pub async fn remove(&mut self, id: Item::Id) {
    if let Some(item) = self.items.write().await.remove(&id) {
      let interval = item.get_interval();
      let mut intervals = self.intervals.write().await;

      if let Some(set) = intervals.get_mut(&interval) {
        if set.remove(&id) && set.is_empty() {
          intervals.remove(&interval);
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use tokio::sync::RwLockReadGuard;

  use super::*;

  #[derive(Debug, PartialEq)]
  struct Task {
    id: i64,
    interval: i64,
    updated: bool,
  }

  impl<Item: Schedulable> Schedule<Item> {
    pub async fn items_ref(&self) -> RwLockReadGuard<'_, HashMap<Item::Id, Arc<Item>>> {
      self.items.read().await
    }

    pub async fn intervals_ref(
      &self,
    ) -> RwLockReadGuard<'_, HashMap<Item::Interval, HashSet<Item::Id>>> {
      self.intervals.read().await
    }
  }

  impl From<(i64, i64)> for Task {
    fn from(args: (i64, i64)) -> Self {
      Task {
        id: args.0,
        interval: args.1,
        updated: false,
      }
    }
  }

  impl Schedulable for Task {
    type Id = i64;
    type Interval = i64;

    fn get_id(&self) -> Self::Id {
      self.id
    }

    fn get_interval(&self) -> Self::Interval {
      self.interval
    }
  }

  #[tokio::test]
  async fn empty_schedule() {
    let schedule: Schedule<Task> = Schedule::new();

    assert!(
      schedule.items_ref().await.is_empty(),
      "schedule items shouldn't be empty"
    );
    assert!(
      schedule.intervals_ref().await.is_empty(),
      "schedule intervals shouldn't be empty"
    );
  }

  #[tokio::test]
  async fn test_empty_schedule() {
    let schedule: Schedule<Task> = Schedule::new();

    assert!(
      schedule.get_due(1, 100).await.is_empty(),
      "empty schedule shouldn't return due items"
    );
  }

  #[tokio::test]
  async fn get_due_on_boundary() {
    let schedule: Schedule<Task> = Schedule::new();

    schedule.insert(Task::from((1, 10))).await;

    assert_eq!(
      schedule.get_due(1, 10).await.len(),
      1,
      "schedule should return item on boundary"
    );
    assert_eq!(
      schedule.get_due(10, 10).await.len(),
      1,
      "schedule should return item on boundary equals"
    );
  }

  #[tokio::test]
  async fn get_due_before_boundary() {
    let schedule: Schedule<Task> = Schedule::new();

    schedule.insert(Task::from((1, 10))).await;

    assert!(
      schedule.get_due(1, 9).await.is_empty(),
      "schedule shouldn't return due items before boundary"
    );
  }

  #[tokio::test]
  async fn test_multiple_intervals() {
    let schedule: Schedule<Task> = Schedule::new();

    schedule.insert(Task::from((1, 5))).await;
    schedule.insert(Task::from((2, 10))).await;

    let ids: Vec<i64> = schedule.get_due(1, 10).await.iter().map(|t| t.id).collect();

    assert!(
      ids.contains(&1),
      "schedule should return item with interval 5"
    );
    assert!(
      ids.contains(&2),
      "schedule should return item with interval 10"
    );
  }

  #[tokio::test]
  async fn test_skip_multiple_intervals() {
    let schedule: Schedule<Task> = Schedule::new();

    schedule.insert(Task::from((1, 10))).await;

    assert_eq!(
      schedule.get_due(1, 35).await.len(),
      1,
      "schedule should return due item even if multiple intervals were passed"
    );
  }

  #[tokio::test]
  async fn insert_single_item_into_schedule() {
    let schedule: Schedule<Task> = Schedule::new();

    schedule.insert(Task::from((1, 30))).await;

    assert!(
      schedule.items_ref().await.contains_key(&1),
      "schedule items should contain entry"
    );
    assert!(
      schedule.intervals_ref().await.contains_key(&30),
      "schedule intervals should contain entry"
    );
    assert_eq!(
      schedule.get(1).await,
      Some(Arc::new(Task::from((1, 30)))),
      "schedule should return entry by id"
    );
  }

  #[tokio::test]
  async fn insert_multiple_items_into_schedule() {
    let schedule: Schedule<Task> = Schedule::new();

    schedule.insert(Task::from((1, 30))).await;
    schedule.insert(Task::from((2, 30))).await;

    assert!(
      schedule.items_ref().await.contains_key(&1),
      "schedule items should contain entry"
    );
    assert!(
      schedule.items_ref().await.contains_key(&2),
      "schedule items should contain entry"
    );
    assert!(
      schedule.intervals_ref().await.contains_key(&30),
      "schedule intervals should contain entry"
    );
    assert_eq!(
      schedule.get(1).await,
      Some(Arc::new(Task::from((1, 30)))),
      "schedule should return entry by id"
    );
    assert_eq!(
      schedule.get(2).await,
      Some(Arc::new(Task::from((2, 30)))),
      "schedule should return entry by id"
    );
  }

  #[tokio::test]
  async fn insert_the_sane_item_twice() {
    let schedule: Schedule<Task> = Schedule::new();

    schedule.insert(Task::from((1, 30))).await;
    schedule.insert(Task::from((1, 30))).await;

    assert_eq!(
      schedule.items_ref().await.len(),
      1,
      "schedule items shouldn't be empty"
    );
    assert_eq!(
      schedule.intervals_ref().await.len(),
      1,
      "schedule intervals shouldn't be empty"
    );
  }

  #[tokio::test]
  async fn remove_item_from_schedule() {
    let mut schedule: Schedule<Task> = Schedule::new();

    schedule.insert(Task::from((1, 30))).await;
    schedule.remove(1).await;

    assert!(
      schedule.items_ref().await.is_empty(),
      "schedule items should be empty"
    );
    assert!(
      schedule.intervals_ref().await.is_empty(),
      "schedule intervals should be empty"
    );
  }
}
