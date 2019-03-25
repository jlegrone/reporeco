use rand::Rng;
use std::collections::HashMap;
use std::{thread, time};

use graphql_client::*;

use crate::UserRepoInteractions;

#[derive(GraphQLQuery)]
#[graphql(
  schema_path = "graphql/schema.json",
  query_path = "graphql/queries/user_stars.graphql",
  response_derives = "Debug"
)]
struct UserStars;

#[derive(GraphQLQuery)]
#[graphql(
  schema_path = "graphql/schema.json",
  query_path = "graphql/queries/user_stars.graphql",
  response_derives = "Debug"
)]
struct UserStarsWithPagination;

#[derive(GraphQLQuery)]
#[graphql(
  schema_path = "graphql/schema.json",
  query_path = "graphql/queries/repository.graphql",
  response_derives = "Debug"
)]
struct RepositoryID;

#[derive(GraphQLQuery)]
#[graphql(
  schema_path = "graphql/schema.json",
  query_path = "graphql/queries/repository.graphql",
  response_derives = "Debug"
)]
struct RepositoryName;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
  github_api_token: String,
}

impl Config {
  pub fn get_repo_name(&self, repo_id: String) -> String {
    let response_body: Response<repository_name::ResponseData> = reqwest::Client::new()
      .post("https://api.github.com/graphql")
      .bearer_auth(&self.github_api_token)
      .json(&RepositoryName::build_query(repository_name::Variables {
        repo_id,
      }))
      .send()
      .expect("no response")
      .json()
      .expect("no response body");

    let repo: repository_name::RepositoryNameNodeOn = response_body
      .data
      .expect("no response body data")
      .node
      .expect("no repository")
      .on;

    if let repository_name::RepositoryNameNodeOn::Repository(repo) = repo {
      return repo.name_with_owner;
    } else {
      return "not found".to_owned();
    }
  }
  pub fn get_repo_id(&self, owner: String, name: String) -> String {
    let response_body: Response<repository_id::ResponseData> = reqwest::Client::new()
      .post("https://api.github.com/graphql")
      .bearer_auth(&self.github_api_token)
      .json(&RepositoryID::build_query(repository_id::Variables {
        owner,
        name,
      }))
      .send()
      .expect("no response")
      .json()
      .expect("no response body");

    response_body
      .data
      .expect("no response body data")
      .repository
      .expect("no repository")
      .id
  }
  pub fn get_data(
    &self,
    min_followers: u32,
    users_per_page: u8,
    stars_per_user: u8,
    max_pages: u16,
  ) -> Result<UserRepoInteractions, failure::Error> {
    let client = reqwest::Client::new();
    let mut interactions: UserRepoInteractions = HashMap::new();
    let mut cursor: Option<String> = None;
    let default_backoff = time::Duration::from_millis(rand::thread_rng().gen_range(100, 200));
    let backoff_increment = time::Duration::from_millis(5000);
    let mut backoff = default_backoff;

    for _ in 0..max_pages {
      let search = format!("followers:{}..{}", min_followers, min_followers + 1);
      let res = match &cursor {
        Some(cursor) => client
          .post("https://api.github.com/graphql")
          .bearer_auth(&self.github_api_token)
          .json(&UserStarsWithPagination::build_query(
            user_stars_with_pagination::Variables {
              search,
              max_users: users_per_page as i64,
              max_stars: stars_per_user as i64,
              cursor: cursor.to_string(),
            },
          ))
          .send(),
        None => client
          .post("https://api.github.com/graphql")
          .bearer_auth(&self.github_api_token)
          .json(&UserStars::build_query(user_stars::Variables {
            search,
            max_users: users_per_page as i64,
            max_stars: stars_per_user as i64,
          }))
          .send(),
      };

      let response_body: Response<user_stars::ResponseData> = match res {
        Ok(mut resp) => resp.json()?,
        Err(e) => {
          warn!("An error: {}; skipped.", e);
          backoff += backoff_increment;
          thread::sleep(backoff);
          continue;
        }
      };

      info!("{:?}", response_body);

      if let Some(errors) = response_body.errors {
        println!("there are errors:");
        for error in &errors {
          println!("{:?}", error);
        }
      }

      if response_body.data.is_none() {
        println!("missing response data");
        backoff += backoff_increment;
        thread::sleep(backoff);
        continue;
      } else {
        backoff = default_backoff;
      }
      let response_data: user_stars::ResponseData =
        response_body.data.expect("missing response data");

      if let Some(limit) = response_data.rate_limit.rate_limit {
        println!(
          "Min followers: {}, Rate limit: {}, Next page: {:?}",
          min_followers, limit.remaining, cursor
        );
      }

      for node in &mut response_data
        .search
        .user_stars
        .nodes
        .expect("missing user nodes")
        .into_iter()
      {
        if let Some(node) = node {
          match node {
            user_stars::UserStarsNodes::User(user) => {
              let repo_ids: Vec<String> = user
                .starred_repositories
                .nodes
                .expect("missing repo nodes")
                .into_iter()
                .map(|r| r.unwrap().id)
                .collect();
              if !repo_ids.is_empty() {
                interactions.insert(user.id.to_string(), repo_ids);
              }
            }
            _ => {
              continue;
            }
          }
        }
      }

      if response_data
        .search
        .user_stars
        .page_info
        .end_cursor
        .is_none()
      {
        break;
      }

      cursor = response_data.search.user_stars.page_info.end_cursor;
      thread::sleep(backoff);
    }

    Ok(interactions)
  }
}
