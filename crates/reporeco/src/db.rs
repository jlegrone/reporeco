use bincode::{deserialize, serialize};
use sled::Db;
use std::str::from_utf8;

use crate::{InteractionStore, UserRepoInteractions};

impl InteractionStore for Db {
  fn save_interactions(self: &Self, interactions: UserRepoInteractions) {
    for (user_id, repo_ids) in interactions.iter() {
      if let Ok(encoded) = serialize(&repo_ids) {
        self.set(user_id, encoded).expect("could not set value");
      }
    }

    // block until all operations are on-disk
    self.flush().expect("could not write")
  }

  fn get_interactions(self: &Self) -> UserRepoInteractions {
    let mut interactions = UserRepoInteractions::new();
    for entry in self.iter() {
      if let Ok((k, v)) = entry {
        interactions.insert(from_utf8(&k).unwrap().to_string(), deserialize(&v).unwrap());
      }
    }
    interactions
  }
}
