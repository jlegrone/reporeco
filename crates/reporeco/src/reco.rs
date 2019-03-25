use recoreco::{
  indicators,
  io::write_indicators,
  stats::{DataDictionary, Renaming},
};
use serde_json::Deserializer;
use std::fs::File;
use std::io::BufReader;

use crate::Interaction;

pub fn generate_indicators(interactions: Vec<Interaction>, path: String) {
  let data_dict = DataDictionary::from(interactions.iter());

  println!(
    "Found {} interactions between {} users and {} items.",
    data_dict.num_interactions(),
    data_dict.num_users(),
    data_dict.num_items(),
  );

  let indicated_items = indicators(interactions.into_iter(), &data_dict, 5, 500, 500);
  let renaming = Renaming::from(data_dict);

  write_indicators(&indicated_items, &renaming, Some(path)).unwrap();
}

#[derive(Deserialize, Debug)]
struct Recommendation {
  for_item: String,
  indicated_items: Vec<String>,
}

pub fn get_indicated_items(repo_id: &str, file: File) -> Option<Vec<String>> {
  let stream = Deserializer::from_reader(BufReader::new(file)).into_iter::<Recommendation>();

  for value in stream {
    if let Ok(recommendation) = value {
      if repo_id == recommendation.for_item {
        return Some(recommendation.indicated_items);
      }
    }
  }

  None
}
