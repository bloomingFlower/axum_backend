-- Create the schema
CREATE TABLE IF NOT EXISTS fast_logger.hnstory (
   id text PRIMARY KEY,
   title text,
   author text,
   url text,
   story_text text,
   tags list<text>,
   points int
);