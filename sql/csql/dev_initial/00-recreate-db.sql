-- Create the Keyspace
CREATE KEYSPACE IF NOT EXISTS fast_logger
WITH REPLICATION = {
  'class': 'NetworkTopologyStrategy',
  'replication_factor': 1
};