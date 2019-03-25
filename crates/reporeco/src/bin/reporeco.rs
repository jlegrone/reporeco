#[macro_use]
extern crate clap;

use clap::{App, AppSettings, Arg, SubCommand};
use reporeco::{gather, get_indicated_items, train};

fn main() {
  let training_data = Arg::with_name("training-data")
    .long("db")
    .value_name("PATH")
    .help("The path at which training data is persisted on disk")
    .default_value("data")
    .required(true);
  let reco_data = Arg::with_name("indicator-data")
    .short("i")
    .long("indicator-data")
    .value_name("PATH")
    .help("The path at which the recommendation data is persisted on disk")
    .default_value("indicators.json")
    .required(true);
  let matches = App::new("reporeco")
    .version(crate_version!())
    .author("Jacob LeGrone <dev@jacob.work>")
    .about("Generate recommendations for Github repositories")
    .setting(AppSettings::SubcommandRequiredElseHelp)
    .subcommand(
      SubCommand::with_name("gather")
        .about("Gather starred repository user data from Github")
        .arg(&training_data)
        .arg(
          Arg::with_name("min-followers")
            .long("min-followers")
            .short("m")
            .value_name("NUMBER")
            .default_value("20")
            .help("minimum number of followers for users to be scraped"),
        )
        .arg(
          Arg::with_name("max-followers")
            .long("max-followers")
            .short("M")
            .value_name("NUMBER")
            .default_value("50")
            .help("maximum number of followers for users to be scraped"),
        )
        .arg(
          Arg::with_name("users-per-page")
            .long("users-per-page")
            .value_name("NUMBER")
            .default_value("5")
            .help("number of users per graphql request"),
        )
        .arg(
          Arg::with_name("stars-per-user")
            .long("stars-per-user")
            .value_name("NUMBER")
            .default_value("100")
            .help("maximum number of stars to be processed per user"),
        )
        .arg(
          Arg::with_name("max-pages")
            .long("max-pages")
            .short("p")
            .value_name("NUMBER")
            .default_value("200")
            .help("maximum number of graphql requests to make per user search"),
        ),
    )
    .subcommand(
      SubCommand::with_name("train")
        .about("Generate recommendations from existing user data")
        .arg(&training_data)
        .arg(&reco_data),
    )
    .subcommand(
      SubCommand::with_name("recommend")
        .about("Get recommendations for a repository")
        .arg(&reco_data)
        .arg(
          Arg::with_name("repository")
            .help("Github repository for which to get recommendations, eg. 'kubernetes/kubernetes'")
            .required(true)
            .index(1),
        ),
    )
    .get_matches();

  if let Some(matches) = matches.subcommand_matches("gather") {
    gather(
      matches
        .value_of("min-followers")
        .unwrap()
        .parse()
        .expect("could not parse min-followers"),
      matches
        .value_of("max-followers")
        .unwrap()
        .parse()
        .expect("could not parse max-followers"),
      matches
        .value_of("users-per-page")
        .unwrap()
        .parse()
        .expect("could not parse users-per-page"),
      matches
        .value_of("stars-per-user")
        .unwrap()
        .parse()
        .expect("could not parse stars-per-user"),
      matches
        .value_of("max-pages")
        .unwrap()
        .parse()
        .expect("could not parse max-pages"),
      matches.value_of("training-data").unwrap(),
    );
  } else if let Some(matches) = matches.subcommand_matches("train") {
    train(
      matches.value_of("training-data").unwrap(),
      matches.value_of("indicator-data").unwrap(),
    );
  } else if let Some(matches) = matches.subcommand_matches("recommend") {
    let repo = matches.value_of("repository").unwrap();
    if let Some(items) = get_indicated_items(repo, matches.value_of("indicator-data").unwrap()) {
      items
        .iter()
        .map(|repo| format!("https://github.com/{}", repo))
        .for_each(|repo| println!("{}", repo));
    } else {
      println!("No recommendations found for {}", repo);
    }
  }
}
