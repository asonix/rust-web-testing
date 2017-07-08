use models::user::{User, CreateUser};
use auth_result::AuthResult;
use rocket_contrib::JSON;

#[derive(Deserialize)]
pub struct Token {
    token: String,
}

#[post("/sign-up", format = "application/json", data = "<create_user>")]
pub fn sign_up(create_user: JSON<CreateUser>) -> AuthResult {
    let new_user = create_user.0.insertable()?;

    let user = new_user.save()?;

    AuthResult::user_created(user)
}

#[post("/log-in", format = "application/json", data = "<create_user>")]
pub fn log_in(create_user: JSON<CreateUser>) -> AuthResult {
    let user = create_user.0.authenticate()?;

    let token = user.create_webtoken().ok();

    AuthResult::authenticated(token)
}

#[post("/is-authenticated", format = "application/json", data = "<token>")]
pub fn is_authenticated(token: JSON<Token>) -> AuthResult {
    User::from_webtoken(token.0.token)?;

    AuthResult::authenticated(None)
}

#[get("/verify/<verification_token>")]
pub fn verify(verification_token: String) -> AuthResult {
    let user = User::verify_with_code(verification_token)?;

    let token = user.create_webtoken()?;

    AuthResult::user_verified(token)
}
