use actix_web::HttpResponse;
use bcrypt::hash;
use serde::Serialize;
use tokio_postgres::{Error, GenericClient, Row};

#[derive(serde::Serialize)]
pub struct Product {
    pub name: String
}

#[derive(serde::Serialize)]
pub struct Customer {
    pub id: i32,
    pub username: String,
    pub email: String
}

#[derive(Serialize)]
struct DatabaseError {
    message: String,
    hint: String
}

impl Customer {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            username: row.get("username"),
            email: row.get("email")
        }
    }

    pub async fn all<C: GenericClient>(client: &C) -> Result<Vec<Customer>, Error> {
        let stmt = client.prepare("SELECT id, username, email FROM customer").await?;
        let rows = client.query(&stmt, &[]).await?;
        Ok(rows.into_iter().map(Customer::from).collect())
    }

    pub async fn register<C: GenericClient>(client: &C, username: String, email: String, password: String) -> HttpResponse {
        let password_hash = match hash(password.clone(), 11) {
            Ok(hash) => hash,
            Err(..) => return HttpResponse::InternalServerError().json("Failed to hash password")
        };
        match client.query("CALL register($1,$2,$3)", &[&username, &email, &password_hash]).await {
            Ok(..) => HttpResponse::Ok().into(),
            Err(err) => {
                match err.as_db_error() {
                    Some(err) => HttpResponse::BadRequest().json(DatabaseError {
                        message: err.message().to_string(),
                        hint: err.hint().unwrap_or("").to_string()
                    }),
                    None => HttpResponse::InternalServerError().into()
                }
            }
        }
    }
}
