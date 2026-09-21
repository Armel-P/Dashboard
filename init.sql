CREATE TABLE users (
  id UUID PRIMARY KEY,
  mail VARCHAR(255) NOT NULL UNIQUE,
  name VARCHAR(255) NOT NULL,
  password_hash TEXT NOT NULL,
  dashboard_map JSONB,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE refresh_tokens (
  id UUID PRIMARY KEY REFERENCES users(id),
  token_hash TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE oauth_connections (
  id UUID PRIMARY KEY,
  user_id UUID REFERENCES users(id) NOT NULL,
  provider VARCHAR(255) NOT NULL,
  provider_user_id TEXT NOT NULL,
  access_token TEXT NOT NULL,
  refresh_token TEXT NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  metadata JSONB NOT NULL
);
