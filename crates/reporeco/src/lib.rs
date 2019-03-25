#![warn(rust_2018_idioms)]
#![warn(clippy::all)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

mod db;
mod github;
mod reco;

use envy;
use rayon::prelude::*;
use sled::Db;
use std::fs::File;

type Interaction = (String, String);

type UserRepoInteractions = std::collections::HashMap<String, Vec<String>>;

trait InteractionStore {
  fn save_interactions(&self, interactions: UserRepoInteractions);
  fn get_interactions(&self) -> UserRepoInteractions;
}

pub fn gather(
  min_followers: u32,
  max_followers: u32,
  users_per_page: u8,
  stars_per_user: u8,
  max_pages: u16,
  db_path: &str,
) {
  dotenv::dotenv().ok();
  env_logger::init();
  let config: github::Config = envy::from_env().unwrap();

  let store = Db::start_default(db_path).expect("could not start sled");
  let count = store.len();

  let sequence: Vec<u32> = (0..(max_followers - min_followers)).collect();

  sequence.par_iter().for_each(|i| {
    let interactions = config
      .get_data(min_followers + i, users_per_page, stars_per_user, max_pages)
      .expect("no data");
    store.save_interactions(interactions);
  });

  println!("Collected data on {} users.", store.len() - count);
}

pub fn train(gh_data_path: &str, reco_data_path: &str) {
  let mut interactions: Vec<Interaction> = Vec::new();
  let store = Db::start_default(gh_data_path).expect("could not start sled");

  for (user_id, repo_ids) in store.get_interactions() {
    let user_id = &user_id;
    for repo_id in repo_ids {
      interactions.push((user_id.to_string(), repo_id));
    }
  }

  reco::generate_indicators(interactions, reco_data_path.to_string());
}

pub fn get_indicated_items(repo_name: &str, reco_data_path: &str) -> Option<Vec<String>> {
  dotenv::dotenv().ok();
  env_logger::init();
  let config: github::Config = envy::from_env().unwrap();

  let indications = File::open(reco_data_path).expect("Could not open file");

  let mut repo_parts = repo_name.split("/");
  let repo_id = config.get_repo_id(
    repo_parts.next().expect("missing repo owner").to_string(),
    repo_parts.next().expect("missing repo name").to_string(),
  );
  let items = reco::get_indicated_items(&repo_id, indications);

  match items {
    Some(items) => Some(
      items
        .par_iter()
        .map(|repo_id| config.get_repo_name(repo_id.to_string()))
        .collect(),
    ),
    None => None,
  }
}
