CREATE TABLE user (
  id UUID PRIMARY KEY,
  mail VARCHAR(255) NOT NULL UNIQUE,
  name VARCHAR(255) NOT NULL,
  password_hash TEXT NOT NULL,
  dashboard_map JSONB
  created_at TIME NOT NULL,
);

CREATE TABLE refresh_token (
  id UUID PRIMARY KEY REFERENCES users(id),
  token_hashed TEXT NOT NULL,
  created_at TIME NOT NULL,
);

CREATE TABLE oath_connection (
  id UUID PRIMARY KEY,
  user_id UUID REFERENCES users(id) NOT NULL,
  provider VARCHAR(255) NOT NULL,
  provider_user_id TEXT NOT NULL,
  access_token TEXT NOT NULL,
  refresh_token TEXT NOT NULL,
  expires_at TIME NOT NULL,
  metadata JSONB NOT NULL
);
