use diesel::r2d2::PooledConnection;
use uuid::Uuid;
use chrono::{ NaiveDateTime, Utc };
use diesel::PgConnection;
use diesel::r2d2::{ ConnectionManager, Pool };
use diesel::{ ExpressionMethods, Insertable, Queryable, RunQueryDsl };
use diesel::query_dsl::methods::{ FilterDsl };
use actix_web::{ get, post, delete, HttpResponse };
use actix_web::web::{ Path, Data };
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use crate::constants::APPLICATION_JSON;
use super::schema::likes;

#[table_name = "likes"]
#[derive(Queryable, Insertable, Deserialize, Serialize)]
pub struct Like {
    pub id: Uuid,
    pub created_at: NaiveDateTime,
    pub tweet_id: Uuid,
}
impl Like {
  pub fn new(tweet_id: Uuid) -> Self {
    Self {
        id: Uuid::new_v4(),
        created_at: Utc::now().naive_utc(),
        tweet_id,
    }
  }
}

#[get("/tweets/{id}/likes")]
pub async fn get_likes_by_tweet(path: Path<(String,)>, pool: Data<Pool<ConnectionManager<PgConnection>>>) -> HttpResponse {
  let t_id = &path.0.0; // tweet id desde los parametros de la url
  let t_id_uuid = Uuid::from_str(t_id); // tweet id formateado a uuid

  if t_id_uuid.is_err() {
    println!("tweet id inválido, error: {:?}", t_id_uuid.err());
    // si no pudimos convertir a un uuid válido, asumimos que el tweet no existe.
    return HttpResponse::NotFound().await.unwrap();
  }

  let conn = pool.get().expect("No pude obtener conexión a la base de datos");
  let response = list_likes(&conn, t_id_uuid.unwrap());

  HttpResponse::Ok()
    .content_type(APPLICATION_JSON)
    .json(response)
}

fn list_likes(conn: &PooledConnection<ConnectionManager<PgConnection>>, t_id_uuid: Uuid) -> Vec<Like> {
  use crate::schema::likes::dsl::*;

  let result = likes
    .filter(tweet_id.eq(t_id_uuid))
    .load::<Like>(conn);
  
  match result {
      Ok(rows) => rows,
      Err(_) => vec![],
  }
}

#[post("/tweets/{id}/likes")]
pub async fn like_tweet(path: Path<(String,)>, pool: Data<Pool<ConnectionManager<PgConnection>>>) -> HttpResponse {
  use crate::schema::likes::dsl::*;

  let t_id = &path.0.0;
  let conn = pool.get().expect("No pude obtener conexión a la base de datos");

  let like = Like::new(Uuid::from_str(t_id).unwrap());
  diesel::insert_into(likes).values(&like).execute(&conn).unwrap();

  HttpResponse::Created()
    .content_type(APPLICATION_JSON)
    .json(&like)
}

#[delete("/tweets/{id}/likes")]
pub async fn remove_like(path: Path<(String,)>, pool: Data<Pool<ConnectionManager<PgConnection>>>) -> HttpResponse {
  use crate::schema::likes::dsl::*;

  let t_id = &path.0.0; // tweet id desde los parametros de la url
  let t_id_uuid = Uuid::from_str(t_id); // tweet id formateado a uuid

  if t_id_uuid.is_err() {
    println!("tweet id inválido, error: {:?}", t_id_uuid.err());
    // si no pudimos convertir a un uuid válido, asumimos que el tweet no existe.
    return HttpResponse::NotFound().await.unwrap();
  }

  let conn = pool.get().expect("No pude obtener conexión a la base de datos");
  let likes_vector = list_likes(&conn, t_id_uuid.unwrap());
  if likes_vector.is_empty() {
    return  HttpResponse::NoContent().content_type(APPLICATION_JSON).await.unwrap()
  }
  let like_to_delete = likes_vector.first();
  let res = diesel::delete(likes.filter(tweet_id.eq(like_to_delete.unwrap().id))).execute(&conn);
  
  match res {
      Err(err) => println!("Error eliminando like, error: {}", err),
      _ => ()
  };

  HttpResponse::NoContent()
    .content_type(APPLICATION_JSON)
    .await
    .unwrap()
}
