#[macro_use]
extern crate diesel;

use actix_web::{App, HttpServer };
use std::env;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use dotenv::dotenv;
mod tweets;
mod likes;
mod constants;
mod schema;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  dotenv().ok();
  
  let database_url =env::var("DATABASE_URL").expect("DATABASE_URL env var no encontrada");
  let manager = ConnectionManager::<PgConnection>::new(database_url);
  let pool = Pool::builder().build(manager).expect("No pude crear el pool");

  HttpServer::new(move|| {
    App::new()
      .data(pool.clone())
      .service(tweets::get_tweets)
      .service(tweets::create_tweet)
      .service(tweets::get_tweet_by_id)
      .service(likes::remove_like)
      .service(likes::like_tweet)
      .service(likes::get_likes_by_tweet)
  })
  .bind("127.0.0.1:8000")?
  .run()
  .await
}
