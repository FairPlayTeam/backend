CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ----------------------------
-- Users (auth)
-- ----------------------------
CREATE TABLE public.users (
  id uuid NOT NULL DEFAULT uuid_generate_v4(),
  email text NOT NULL UNIQUE,
  password_hash text NOT NULL, -- hashed password
  created_at timestamp with time zone DEFAULT now(),
  is_active boolean DEFAULT true,
  last_login timestamp with time zone,
  CONSTRAINT users_pkey PRIMARY KEY (id)
);

-- ----------------------------
-- Private user accounts (profile)
-- ----------------------------
CREATE TABLE public.user_accounts (
  id uuid NOT NULL, -- use same UUID as users
  username text NOT NULL UNIQUE,
  created_at timestamp with time zone DEFAULT now(),
  is_moderator boolean DEFAULT false,
  is_admin boolean DEFAULT false,
  support_balance numeric DEFAULT 0, -- donation total (estimated)
  CONSTRAINT user_accounts_pkey PRIMARY KEY (id),
  CONSTRAINT user_accounts_user_id_fkey FOREIGN KEY (id) REFERENCES public.users(id)
);

-- ----------------------------
-- Public channels
-- ----------------------------
CREATE TABLE public.public_channels (
  id uuid NOT NULL DEFAULT uuid_generate_v4(),
  user_id uuid NOT NULL, -- channel's owner id
  display_name text NOT NULL,
  handle text NOT NULL, -- unique handle (@handle)
  avatar_url text,
  banner_url text,
  created_at timestamp with time zone DEFAULT now(),
  CONSTRAINT public_channels_pkey PRIMARY KEY (id),
  CONSTRAINT public_channels_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.user_accounts(id),
  CONSTRAINT public_channels_handle_unique UNIQUE (handle)
  CONSTRAINT public_channels_unique UNIQUE (user_id) -- one channel per user
);

-- ----------------------------
-- Videos
-- ----------------------------
-- enum for status
DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'video_status') THEN
    CREATE TYPE video_status AS ENUM ('pending','accepted','refused');
  END IF;
END $$;

CREATE TABLE public.videos (
  id uuid NOT NULL DEFAULT uuid_generate_v4(),
  channel_id uuid NOT NULL, -- video's channel
  title text NOT NULL,
  description text,
  storage_path text,
  thumbnail text,
  duration interval, -- or "text" type for formatted duration
  themes text[],
  created_at timestamp with time zone DEFAULT now(),
  quality_score double precision DEFAULT 0,
  
  status video_status NOT NULL DEFAULT 'pending',
  refusal_reason text, -- reason for refusal if status = 'refused'

  is_private boolean DEFAULT false,
  allow_download boolean DEFAULT false,
  
  CONSTRAINT videos_pkey PRIMARY KEY (id),
  CONSTRAINT videos_channel_id_fkey FOREIGN KEY (channel_id) REFERENCES public.public_channels(id)
);

-- ----------------------------
-- Subscriptions
-- ----------------------------
CREATE TABLE public.subscriptions (
  id uuid NOT NULL DEFAULT uuid_generate_v4(),
  subscriber_id uuid NOT NULL, -- account id of the subscriber (private account)
  channel_id uuid NOT NULL, -- creator's public channel id
  notifications boolean DEFAULT true,
  no_ping boolean DEFAULT false,
  created_at timestamp with time zone DEFAULT now(),
  CONSTRAINT subscriptions_pkey PRIMARY KEY (id),
  CONSTRAINT subscriptions_subscriber_fkey FOREIGN KEY (subscriber_id) REFERENCES public.user_accounts(id),
  CONSTRAINT subscriptions_channel_fkey FOREIGN KEY (channel_id) REFERENCES public.public_channels(id),
  CONSTRAINT subscriptions_unique UNIQUE (subscriber_id, channel_id)
);

-- ----------------------------
-- Ratings
-- ----------------------------
CREATE TABLE public.ratings (
  id uuid NOT NULL DEFAULT uuid_generate_v4(),
  video_id uuid NOT NULL,
  user_id uuid NOT NULL,
  score smallint NOT NULL CHECK (score >= 1 AND score <= 5),
  created_at timestamp with time zone DEFAULT now(),
  CONSTRAINT ratings_pkey PRIMARY KEY (id),
  CONSTRAINT ratings_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.user_accounts(id),
  CONSTRAINT ratings_video_id_fkey FOREIGN KEY (video_id) REFERENCES public.videos(id),
  CONSTRAINT ratings_unique UNIQUE (video_id, user_id)
);

-- ----------------------------
-- Donations
-- ----------------------------
CREATE TABLE public.donations (
  id uuid NOT NULL DEFAULT uuid_generate_v4(),
  supporter_id uuid NOT NULL, -- account id of the supporter (private account)
  channel_id uuid NOT NULL, -- creator's public channel id
  amount numeric NOT NULL CHECK (amount > 0),
  method text CHECK (method = ANY (ARRAY['donation', 'ad'])), -- donation or ad revenue
  created_at timestamp with time zone DEFAULT now(),
  CONSTRAINT donations_pkey PRIMARY KEY (id),
  CONSTRAINT donations_supporter_fkey FOREIGN KEY (supporter_id) REFERENCES public.user_accounts(id),
  CONSTRAINT donations_channel_fkey FOREIGN KEY (channel_id) REFERENCES public.public_channels(id)
);
