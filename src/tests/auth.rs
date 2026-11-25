use std::sync::{Arc, atomic::Ordering};

use axum::{
    extract::FromRequestParts,
    http::{Request, StatusCode, header::AUTHORIZATION, request::Parts},
};
use chrono::Duration;

use crate::{
    application::{
        ports::{JwtEncoderPort, PasswordHasherPort, repositories::UserRepository},
        use_cases::auth::{
            GenerateAuthUseCaseResponse, LoginUseCase, LoginUseCaseParams, LogoutUseCase,
            LogoutUseCaseParams, RefreshTokenUseCase, RefreshTokenUseCaseParams, RegisterUseCase,
            RegisterUseCaseParams,
        },
    },
    domain::value_objects::{Error, Jwt, Role, TokenType},
    infrastructure::{
        adapters::{Argon2PasswordHasher, JwtEncoder},
        bootstrap::AppState,
        http::extractors::{AdminUser, AuthUser},
    },
    tests::support::{
        FakePasswordHasher, InMemoryUserRepository, PASSWORD, app_state, jwt_encoder, user,
    },
};

struct Context {
    users: Arc<InMemoryUserRepository>,
    hasher: Arc<FakePasswordHasher>,
}

impl Context {
    fn new() -> Self {
        Self {
            users: Arc::new(InMemoryUserRepository::with(vec![user("u1", "alice")])),
            hasher: Arc::new(FakePasswordHasher::default()),
        }
    }

    async fn register(
        &self,
        username: &str,
        password: &str,
    ) -> Result<GenerateAuthUseCaseResponse, Error> {
        RegisterUseCase {
            user_repository: self.users.clone(),
            jwt_encoder: jwt_encoder(),
            password_hasher: self.hasher.clone(),
        }
        .execute(RegisterUseCaseParams {
            username: username.to_string(),
            password: password.to_string(),
        })
        .await
    }

    async fn login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<GenerateAuthUseCaseResponse, Error> {
        LoginUseCase {
            jwt_encoder: jwt_encoder(),
            user_repository: self.users.clone(),
            password_hasher: self.hasher.clone(),
        }
        .execute(LoginUseCaseParams {
            username: username.to_string(),
            password: password.to_string(),
        })
        .await
    }

    async fn refresh(&self, token: &str) -> Result<GenerateAuthUseCaseResponse, Error> {
        RefreshTokenUseCase {
            jwt_encoder: jwt_encoder(),
            user_repository: self.users.clone(),
        }
        .execute(RefreshTokenUseCaseParams {
            refresh_token: token.to_string(),
        })
        .await
    }

    async fn logout(&self, user_id: &str) -> Result<(), Error> {
        LogoutUseCase {
            user_repository: self.users.clone(),
        }
        .execute(LogoutUseCaseParams {
            user_id: user_id.to_string(),
        })
        .await
    }
}

#[tokio::test]
async fn register_creates_user_and_returns_tokens() {
    let context = Context::new();
    let tokens = context.register("bob", PASSWORD).await.unwrap();

    let access = jwt_encoder().decode(&tokens.access).unwrap();
    let refresh = jwt_encoder().decode(&tokens.refresh).unwrap();
    let stored = context.users.get(&access.sub).unwrap();

    assert_eq!(access.token_type, TokenType::Access);
    assert_eq!(refresh.token_type, TokenType::Refresh);
    assert_eq!(refresh.sub, access.sub);
    assert!(refresh.exp > access.exp);
    assert_eq!(stored.username, "bob");
    assert_eq!(stored.password_hash, Some(format!("hash:{PASSWORD}")));
}

#[tokio::test]
async fn register_rejects_invalid_input() {
    let context = Context::new();
    let short_username = context.register("b", PASSWORD).await.err().unwrap();
    let short_password = context.register("bob", "short").await.err().unwrap();

    assert_eq!(short_username.code, StatusCode::BAD_REQUEST);
    assert_eq!(short_password.code, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn register_rejects_taken_username_case_insensitively() {
    let context = Context::new();
    let error = context.register("ALICE", PASSWORD).await.err().unwrap();
    assert_eq!(error.code, StatusCode::CONFLICT);
}

#[tokio::test]
async fn login_with_valid_credentials() {
    let context = Context::new();
    let tokens = context.login("Alice", PASSWORD).await.unwrap();
    assert_eq!(jwt_encoder().decode(&tokens.access).unwrap().sub, "u1");
}

#[tokio::test]
async fn login_with_wrong_password_is_unauthorized() {
    let context = Context::new();
    let error = context
        .login("alice", "wrong-password")
        .await
        .err()
        .unwrap();
    assert_eq!(error.code, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_with_unknown_user_still_hashes_the_password() {
    let context = Context::new();
    let error = context.login("nobody", PASSWORD).await.err().unwrap();

    assert_eq!(error.code, StatusCode::UNAUTHORIZED);
    assert_eq!(context.hasher.hash_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn refresh_issues_new_tokens() {
    let context = Context::new();
    let tokens = context.login("alice", PASSWORD).await.unwrap();
    let refreshed = context.refresh(&tokens.refresh).await.unwrap();
    assert_eq!(jwt_encoder().decode(&refreshed.access).unwrap().sub, "u1");
}

#[tokio::test]
async fn refresh_rejects_access_tokens_and_garbage() {
    let context = Context::new();
    let tokens = context.login("alice", PASSWORD).await.unwrap();

    let with_access = context.refresh(&tokens.access).await.err().unwrap();
    let with_garbage = context.refresh("garbage").await.err().unwrap();

    assert_eq!(with_access.code, StatusCode::UNAUTHORIZED);
    assert_eq!(with_garbage.code, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn logout_revokes_refresh_tokens() {
    let context = Context::new();
    let tokens = context.login("alice", PASSWORD).await.unwrap();

    context.logout("u1").await.unwrap();

    let error = context.refresh(&tokens.refresh).await.err().unwrap();
    assert_eq!(error.code, StatusCode::UNAUTHORIZED);
    assert_eq!(context.users.get("u1").unwrap().token_version, 1);

    let new_tokens = context.login("alice", PASSWORD).await.unwrap();
    assert!(context.refresh(&new_tokens.refresh).await.is_ok());
}

#[tokio::test]
async fn refresh_for_deleted_user_is_unauthorized() {
    let context = Context::new();
    let tokens = context.login("alice", PASSWORD).await.unwrap();
    context.users.remove("u1");

    let error = context.refresh(&tokens.refresh).await.err().unwrap();
    assert_eq!(error.code, StatusCode::UNAUTHORIZED);
}

fn signed(ttl: Duration) -> Jwt {
    Jwt::new("u1", Role::Admin, 3, TokenType::Refresh, ttl)
}

#[test]
fn jwt_roundtrips_claims() {
    let encoder = JwtEncoder::new("secret".to_string());
    let encoded = encoder.encode(&signed(Duration::minutes(5))).unwrap();
    let decoded = encoder.decode(&encoded).unwrap();

    assert_eq!(decoded.sub, "u1");
    assert_eq!(decoded.role, Role::Admin);
    assert_eq!(decoded.token_version, 3);
    assert_eq!(decoded.token_type, TokenType::Refresh);
}

#[test]
fn jwt_rejects_token_signed_with_another_secret() {
    let encoded = JwtEncoder::new("secret".to_string())
        .encode(&signed(Duration::minutes(5)))
        .unwrap();
    assert!(
        JwtEncoder::new("other".to_string())
            .decode(&encoded)
            .is_err()
    );
}

#[test]
fn jwt_rejects_expired_and_malformed_tokens() {
    let encoder = JwtEncoder::new("secret".to_string());
    let expired = encoder.encode(&signed(Duration::hours(-1))).unwrap();

    assert!(encoder.decode(&expired).is_err());
    assert!(encoder.decode("not-a-token").is_err());
}

#[tokio::test]
async fn argon2_verifies_only_the_original_password() {
    let hash = Argon2PasswordHasher.hash(PASSWORD).await.unwrap();

    assert_ne!(hash, PASSWORD);
    assert!(Argon2PasswordHasher.verify(PASSWORD, &hash).await.unwrap());
    assert!(
        !Argon2PasswordHasher
            .verify("password124", &hash)
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn argon2_salts_every_hash() {
    let first = Argon2PasswordHasher.hash(PASSWORD).await.unwrap();
    let second = Argon2PasswordHasher.hash(PASSWORD).await.unwrap();
    assert_ne!(first, second);
}

#[tokio::test]
async fn argon2_rejects_malformed_hash() {
    assert!(Argon2PasswordHasher.verify(PASSWORD, "nope").await.is_err());
}

fn request_parts(authorization: Option<String>) -> Parts {
    let mut request = Request::builder();
    if let Some(value) = authorization {
        request = request.header(AUTHORIZATION, value);
    }
    request.body(()).unwrap().into_parts().0
}

fn bearer(state: &AppState, version: u32, token_type: TokenType) -> Option<String> {
    let jwt = Jwt::new("u1", Role::Admin, version, token_type, Duration::minutes(5));
    Some(format!(
        "Bearer {}",
        state.jwt_encoder.encode(&jwt).unwrap()
    ))
}

fn auth_setup(role: Role) -> (Arc<InMemoryUserRepository>, Arc<AppState>) {
    let mut stored = user("u1", "alice");
    stored.role = role;
    let users = Arc::new(InMemoryUserRepository::with(vec![stored]));
    let state = app_state(users.clone());
    (users, state)
}

async fn extract_auth(state: &Arc<AppState>, parts: &mut Parts) -> Result<AuthUser, StatusCode> {
    AuthUser::from_request_parts(parts, state)
        .await
        .map_err(|response| response.status())
}

#[tokio::test]
async fn auth_user_accepts_valid_access_token_and_caches_it() {
    let (_, state) = auth_setup(Role::User);
    let mut parts = request_parts(bearer(&state, 0, TokenType::Access));

    let auth_user = extract_auth(&state, &mut parts).await.unwrap();

    assert_eq!(auth_user.user_id, "u1");
    assert!(parts.extensions.get::<AuthUser>().is_some());
}

#[tokio::test]
async fn auth_user_reuses_cached_result() {
    let (users, state) = auth_setup(Role::User);
    let mut parts = request_parts(bearer(&state, 0, TokenType::Access));
    extract_auth(&state, &mut parts).await.unwrap();

    users.remove("u1");

    assert_eq!(
        extract_auth(&state, &mut parts).await.unwrap().user_id,
        "u1"
    );
}

#[tokio::test]
async fn auth_user_takes_role_from_stored_user_not_token() {
    let (_, state) = auth_setup(Role::User);
    let mut parts = request_parts(bearer(&state, 0, TokenType::Access));
    assert_eq!(
        extract_auth(&state, &mut parts).await.unwrap().role,
        Role::User
    );
}

#[tokio::test]
async fn auth_user_rejects_token_revoked_by_logout() {
    let (users, state) = auth_setup(Role::User);
    let header = bearer(&state, 0, TokenType::Access);
    users.increment_token_version("u1").await.unwrap();

    let result = extract_auth(&state, &mut request_parts(header)).await;
    assert_eq!(result.err(), Some(StatusCode::UNAUTHORIZED));
}

#[tokio::test]
async fn auth_user_rejects_token_of_deleted_user() {
    let (users, state) = auth_setup(Role::User);
    let header = bearer(&state, 0, TokenType::Access);
    users.remove("u1");

    let result = extract_auth(&state, &mut request_parts(header)).await;
    assert_eq!(result.err(), Some(StatusCode::UNAUTHORIZED));
}

#[tokio::test]
async fn auth_user_rejects_refresh_token() {
    let (_, state) = auth_setup(Role::User);
    let header = bearer(&state, 0, TokenType::Refresh);

    let result = extract_auth(&state, &mut request_parts(header)).await;
    assert_eq!(result.err(), Some(StatusCode::UNAUTHORIZED));
}

#[tokio::test]
async fn auth_user_rejects_missing_or_malformed_header() {
    let (_, state) = auth_setup(Role::User);
    let headers = [
        None,
        Some("Token abc".to_string()),
        Some("Bearer garbage".to_string()),
    ];

    for header in headers {
        let result = extract_auth(&state, &mut request_parts(header)).await;
        assert_eq!(result.err(), Some(StatusCode::UNAUTHORIZED));
    }
}

async fn extract_admin(role: Role) -> Result<AdminUser, StatusCode> {
    let (_, state) = auth_setup(role);
    let mut parts = request_parts(bearer(&state, 0, TokenType::Access));

    AdminUser::from_request_parts(&mut parts, &state)
        .await
        .map_err(|response| response.status())
}

#[tokio::test]
async fn admin_user_allows_admins() {
    assert_eq!(extract_admin(Role::Admin).await.unwrap().user_id, "u1");
}

#[tokio::test]
async fn admin_user_forbids_users_even_if_token_claims_admin() {
    assert_eq!(
        extract_admin(Role::User).await.err(),
        Some(StatusCode::FORBIDDEN)
    );
}
