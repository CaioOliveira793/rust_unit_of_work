
CREATE TABLE public.user (
  id UUID CONSTRAINT user_pk PRIMARY KEY,
  name TEXT NOT NULL,
  email TEXT NOT NULL
);
