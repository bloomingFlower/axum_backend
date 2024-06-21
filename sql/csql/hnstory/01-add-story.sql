-- Add a new story to the hnstory table
INSERT INTO fast_logger.hnstory (
    author,
    id,
    title,
    url,
    story_text,
    tags,
    points)
VALUES (?, ?, ?, ?, ?, ?, ?);
