CREATE TABLE IF NOT EXISTS users (
    user_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS videos (
    video_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    video_path VARCHAR(255) NOT NULL
);