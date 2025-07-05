#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Header {
  Authorization,
  XRequestId,
  XIpAddress,
  XForwardedFor,
  Path,
  UserAgent,
  AcceptLanguage,
  SessionId,
  Token,
  CreatedAt,
  ExpiresAt,
  LastActivityAt,
  UserId,
  DeviceId,
  Roles,
  IsOauth,
  Props,
}

impl Header {
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Authorization => "authorization",
      Self::XRequestId => "x-request-id",
      Self::XIpAddress => "x-ip-address",
      Self::XForwardedFor => "x-forwarded-for",
      Self::Path => "path",
      Self::UserAgent => "user-agent",
      Self::AcceptLanguage => "accept-language",
      Self::SessionId => "session-id",
      Self::Token => "token",
      Self::CreatedAt => "created-at",
      Self::ExpiresAt => "expires-at",
      Self::LastActivityAt => "last-activity-at",
      Self::UserId => "user-id",
      Self::DeviceId => "device-id",
      Self::Roles => "roles",
      Self::IsOauth => "is-oauth",
      Self::Props => "props",
    }
  }
}
