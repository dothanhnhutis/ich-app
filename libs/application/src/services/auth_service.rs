use domain::repositories::UserRepository;
use validator::Validate;

use crate::dto::auth_dto::{LoginRequest, LoginResponse};
use crate::errors::AppError;

pub struct AuthService<R: UserRepository> {
    user_repo: R,
}

impl<R: UserRepository> AuthService<R> {
    pub fn new(user_repo: R) -> Self {
        Self { user_repo }
    }

    pub async fn login(&self, request: LoginRequest) -> Result<LoginResponse, AppError> {
        request
            .validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        let user = self
            .user_repo
            .find_by_email(&request.email)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Email hoặc mật khẩu không đúng".into()))?;

        // 3. Verify password
        // TODO: Dùng argon2 để verify password hash
        // let is_valid = argon2::verify_encoded(&user.password_hash, request.password.as_bytes())
        //     .map_err(|e| AppError::Internal(e.to_string()))?;
        // if !is_valid {
        //     return Err(AppError::Unauthorized("Sai mật khẩu".into()));
        // }

        // 4. TODO: Generate JWT token

        // 5. Return response
        Ok(LoginResponse {
            message: "Login thành công".into(),
            user_id: user.id.to_string(),
        })
    }
}
