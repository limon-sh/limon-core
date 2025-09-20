//! A module for managing scheduled items that are periodically polled.
//!
//! The [`schedule`](crate::schedule) module provides structures and
//! traits to manage objects that need to be executed, updated, or
//! checked at regular intervals. Each item must implement the
//! [`Schedulable`] trait, which defines a unique identifier and
//! an associated interval.
//!
//! The [`Schedule`] struct maintains:
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
//! let mut schedule: Schedule<Task> = Schedule::new();
//!
//! schedule.insert(Task { id: 1, interval: 30 });
//! schedule.insert(Task { id: 2, interval: 60 });
//!
//! assert_eq!(schedule.get_due(0, 90).len(), 2);
//! ```

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// A trait for items that can be scheduled.
///
/// This trait defines the necessary requirements for an item to be
/// stored and managed by a [`Schedule`]. Each item must have a unique
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

/// A schedule for managing [`Schedulable`] items.
///
/// The [`Schedule`] structure stores items indexed by their unique
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
  items: HashMap<Item::Id, Item>,
  intervals: HashMap<Item::Interval, HashSet<Item::Id>>,
}

impl<Item: Schedulable> Schedule<Item> {
  /// Create a new schedule.
  pub fn new() -> Self {
    Self {
      items: HashMap::new(),
      intervals: HashMap::new(),
    }
  }

  /// Get an item by `id`.
  pub fn get(&self, id: Item::Id) -> Option<&Item> {
    self.items.get(&id)
  }

  /// Get mut ref on an item by `id`.
  pub fn get_mut(&mut self, id: Item::Id) -> Option<&mut Item> {
    self.items.get_mut(&id)
  }

  /// Get items that are included in the interval `from` and `to`.
  ///
  /// An element is included in the interval if there is at least
  /// one value between `from` and `to` that is divisible by
  /// the item's [`Interval`](Schedulable::Interval) without a remainder.
  pub fn get_due(&self, from: i64, to: i64) -> Vec<&Item> {
    let mut result = Vec::new();

    for (interval, ids) in &self.intervals {
      let interval = (*interval).into();
      let next_check = ((from / interval) + 1) * interval;

      if next_check <= to {
        for id in ids {
          if let Some(item) = self.items.get(id) {
            result.push(item);
          }
        }
      }
    }

    result
  }

  /// Insert an item into schedule.
  ///
  /// If an item with this `id` is already in the schedule, it will be replaced.
  pub fn insert(&mut self, item: Item) {
    let id = item.get_id();
    let interval = item.get_interval();

    if let Some(ids_set) = self.intervals.get_mut(&interval) {
      ids_set.insert(id);
    } else {
      let mut set = HashSet::new();
      set.insert(id);

      self.intervals.insert(interval, set);
    }

    self.items.insert(id, item);
  }

  /// Remove an item by `id` from the schedule if it exists.
  pub fn remove(&mut self, id: Item::Id) {
    if let Some(item) = self.items.remove(&id) {
      let interval = item.get_interval();

      if let Some(set) = self.intervals.get_mut(&interval) {
        if set.remove(&id) && set.is_empty() {
          self.intervals.remove(&interval);
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Debug, PartialEq)]
  struct Task {
    id: i64,
    interval: i64,
    updated: bool,
  }

  impl<Item: Schedulable> Schedule<Item> {
    #[doc(hidden)]
    pub fn items_ref(&self) -> &HashMap<Item::Id, Item> {
      &self.items
    }

    #[doc(hidden)]
    pub fn intervals_ref(&self) -> &HashMap<Item::Interval, HashSet<Item::Id>> {
      &self.intervals
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

  #[test]
  fn empty_schedule() {
    let schedule: Schedule<Task> = Schedule::new();

    assert!(
      schedule.items_ref().is_empty(),
      "schedule items shouldn't be empty"
    );
    assert!(
      schedule.intervals_ref().is_empty(),
      "schedule intervals shouldn't be empty"
    );
  }

  #[test]
  fn test_empty_schedule() {
    let schedule: Schedule<Task> = Schedule::new();

    assert!(
      schedule.get_due(0, 100).is_empty(),
      "empty schedule shouldn't return due items"
    );
  }

  #[test]
  fn get_due_on_boundary() {
    let mut schedule: Schedule<Task> = Schedule::new();

    schedule.insert(Task::from((1, 10)));

    assert_eq!(
      schedule.get_due(0, 10).len(),
      1,
      "schedule should return item on boundary"
    );
  }

  #[test]
  fn get_due_before_boundary() {
    let mut schedule: Schedule<Task> = Schedule::new();

    schedule.insert(Task::from((1, 10)));

    assert!(
      schedule.get_due(0, 9).is_empty(),
      "schedule shouldn't return due items before boundary"
    );
  }

  #[test]
  fn test_multiple_intervals() {
    let mut schedule: Schedule<Task> = Schedule::new();

    schedule.insert(Task::from((1, 5)));
    schedule.insert(Task::from((2, 10)));

    let ids: Vec<i64> = schedule.get_due(0, 10).iter().map(|t| t.id).collect();

    assert!(
      ids.contains(&1),
      "schedule should return item with interval 5"
    );
    assert!(
      ids.contains(&2),
      "schedule should return item with interval 10"
    );
  }

  #[test]
  fn test_skip_multiple_intervals() {
    let mut schedule: Schedule<Task> = Schedule::new();

    schedule.insert(Task::from((1, 10)));

    assert_eq!(
      schedule.get_due(0, 35).len(),
      1,
      "schedule should return due item even if multiple intervals were passed"
    );
  }

  #[test]
  fn insert_single_item_into_schedule() {
    let mut schedule: Schedule<Task> = Schedule::new();

    schedule.insert(Task::from((1, 30)));

    assert!(
      schedule.items_ref().contains_key(&1),
      "schedule items should contain entry"
    );
    assert!(
      schedule.intervals_ref().contains_key(&30),
      "schedule intervals should contain entry"
    );
    assert_eq!(
      schedule.get(1),
      Some(&Task::from((1, 30))),
      "schedule should return entry by id"
    );
  }

  #[test]
  fn insert_multiple_items_into_schedule() {
    let mut schedule: Schedule<Task> = Schedule::new();

    schedule.insert(Task::from((1, 30)));
    schedule.insert(Task::from((2, 30)));

    assert!(
      schedule.items_ref().contains_key(&1),
      "schedule items should contain entry"
    );
    assert!(
      schedule.items_ref().contains_key(&2),
      "schedule items should contain entry"
    );
    assert!(
      schedule.intervals_ref().contains_key(&30),
      "schedule intervals should contain entry"
    );
    assert_eq!(
      schedule.get(1),
      Some(&Task::from((1, 30))),
      "schedule should return entry by id"
    );
    assert_eq!(
      schedule.get(2),
      Some(&Task::from((2, 30))),
      "schedule should return entry by id"
    );
  }

  #[test]
  fn insert_the_sane_item_twice() {
    let mut schedule: Schedule<Task> = Schedule::new();

    schedule.insert(Task::from((1, 30)));
    schedule.insert(Task::from((1, 30)));

    assert_eq!(
      schedule.items_ref().len(),
      1,
      "schedule items shouldn't be empty"
    );
    assert_eq!(
      schedule.intervals_ref().len(),
      1,
      "schedule intervals shouldn't be empty"
    );
  }

  #[test]
  fn update_item_from_schedule() {
    let mut schedule: Schedule<Task> = Schedule::new();

    schedule.insert(Task::from((1, 30)));
    if let Some(item) = schedule.get_mut(1) {
      item.updated = true;
    }

    assert!(
      schedule.get(1).unwrap().updated,
      "schedule should return mutable reference to the item"
    );
  }

  #[test]
  fn remove_item_from_schedule() {
    let mut schedule: Schedule<Task> = Schedule::new();

    schedule.insert(Task::from((1, 30)));
    schedule.remove(1);

    assert!(
      schedule.items_ref().is_empty(),
      "schedule items should be empty"
    );
    assert!(
      schedule.intervals_ref().is_empty(),
      "schedule intervals should be empty"
    );
  }
}
