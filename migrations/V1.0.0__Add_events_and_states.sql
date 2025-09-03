CREATE TABLE events (
  envelope        jsonb NOT NULL,
  -- Convenience columns for filtering/indexing
  event_id        uuid GENERATED ALWAYS AS ((envelope->>'event_id')::uuid) STORED NOT NULL,
  aggregate_id    uuid GENERATED ALWAYS AS ((envelope->>'aggregate_id')::uuid) STORED NOT NULL,
  aggregate_type  text GENERATED ALWAYS AS (envelope->>'aggregate_type') STORED NOT NULL,
  event_name      text GENERATED ALWAYS AS (envelope->'data'->>'event_name') STORED NOT NULL,
  --
  -- occ_version serves as a OPTIMIZING CONCURRENT VERSIONS and per-aggregate ordering
  --
  -- OPTIMIZING CONCURRENT VERSIONS: Prevents more than 1 user from performing
  -- changes to the aggregate at any one time.
  --
  occ_version     bigint GENERATED ALWAYS AS ((envelope->>'occ_version')::bigint) STORED NOT NULL,
  --
  -- Optional: duplicate timestamp for easier querying
  -- event_ts      timestamptz GENERATED ALWAYS AS ((envelope->>'timestamp')::timestamptz) STORED,

  recorded_at     timestamptz NOT NULL DEFAULT now(),
  UNIQUE (aggregate_id, occ_version)
);

CREATE TABLE states (
  aggregate_id  uuid GENERATED ALWAYS AS ((state->>'aggregate_id')::uuid) STORED PRIMARY KEY NOT NULL,
  aggregate_type  text GENERATED ALWAYS AS (state->>'aggregate_type') STORED NOT NULL,
  --
  -- occ_version serves as a OPTIMIZING CONCURRENT VERSIONS
  --
  occ_version   bigint GENERATED ALWAYS AS ((state->>'occ_version')::bigint) STORED NOT NULL,
  timestamp     timestamptz NOT NULL DEFAULT now(),
  state         jsonb NOT NULL
);
