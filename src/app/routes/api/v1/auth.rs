use crate::api::{ApiResponse, Resp};
use crate::database::Db;
use crate::models::{Account, AccountSummary, NewAccount};
use crate::result::Error;
use argon2::{
    password_hash::{
        rand_core::OsRng, Error as PwdHashError, PasswordHash, PasswordHasher, PasswordVerifier,
        SaltString,
    },
    Argon2,
};
use rocket::serde::json::Json;
use rocket::State;
use rocket_db_pools::diesel::prelude::*;
use rocket_db_pools::Connection;
use serde::Deserialize;

// ------------------------------------------------------------------------------------------ Login

#[derive(Debug, Deserialize)]
pub struct LoginRequest<'r> {
    alias: &'r str,
    password: &'r str,
}

#[post("/api/v1/auth/login", data = "<login_request>")]
pub async fn login<'a, 'b, 'c>(
    mut db: Connection<Db>,
    crypto: &State<Argon2<'a>>,
    login_request: Json<LoginRequest<'b>>,
) -> Resp<'c, &'c str> {
    use crate::schema::accounts::dsl::*;
    let account = accounts
        .filter(alias.eq(login_request.alias))
        .first::<Account>(&mut db)
        .await?;
    let hash = PasswordHash::new(&account.password_hash)?;
    crypto
        .verify_password(login_request.password.as_bytes(), &hash)
        .map_err(|_| Error::NotFound)
        .map(|_| ApiResponse::ok("Connected"))
}

// --------------------------------------------------------------------------------------- Register

#[derive(Debug, Deserialize)]
pub struct RegisterRequest<'r> {
    alias: &'r str,
    email: &'r str,
    password: &'r str,
}

impl<'r> RegisterRequest<'r> {
    pub fn as_new_account(&self, crypto: &Argon2) -> Result<NewAccount, PwdHashError> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = crypto
            .hash_password(self.password.as_bytes(), &salt)?
            .to_string();
        Ok(NewAccount {
            alias: self.alias.into(),
            email: self.email.into(),
            password_hash,
        })
    }
}

#[post("/api/v1/auth/register", data = "<register_request>")]
pub async fn register<'a, 'b, 'c>(
    mut db: Connection<Db>,
    crypto: &State<Argon2<'a>>,
    register_request: Json<RegisterRequest<'b>>,
) -> Result<Json<AccountSummary>, Error<'c>> {
    use crate::schema::accounts::dsl::*;
    let new_account: NewAccount = register_request.as_new_account(crypto)?;
    let account_summary = diesel::insert_into(accounts)
        .values(new_account)
        .returning(AccountSummary::as_returning())
        .get_result(&mut db)
        .await?;
    Ok(Json(account_summary))
}
