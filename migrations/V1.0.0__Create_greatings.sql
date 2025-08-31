create table greetings (
  message text
);

insert into greetings (message)
values ('Hello, World!');


create table events (
  id uuid,
  name text,
  data jsonb
);

CREATE TABLE user_events (
  event_id        uuid GENERATED ALWAYS AS ((envelope->>'event_id')::uuid) STORED,
  aggregate_id    uuid GENERATED ALWAYS AS ((envelope->>'aggregate_id')::uuid) STORED,
  aggregate_type  text GENERATED ALWAYS AS (envelope->>'aggregate_type') STORED,
  occ_version     bigint GENERATED ALWAYS AS ((envelope->>'occ_version')::bigint) STORED,
  recorded_at      timestamptz NOT NULL DEFAULT now(),
  envelope          jsonb NOT NULL,

  -- Convenience columns for filtering/indexing
  event_name       text GENERATED ALWAYS AS (envelope->'data'->>'event_name') STORED

  -- Optional: duplicate timestamp for easier querying
  -- event_ts      timestamptz GENERATED ALWAYS AS ((envelope->>'timestamp')::timestamptz) STORED,

  -- seq           bigint NOT NULL,                     -- per-aggregate ordering
  -- UNIQUE (aggregate_id, seq)
);

-- Indexes for common queries
-- CREATE INDEX user_events_by_agg_seq ON user_events (aggregate_id, seq);
-- CREATE INDEX user_events_by_type    ON user_events (event_type);
-- CREATE INDEX user_events_recorded   ON user_events (recorded_at);
--
-- -- If you’ll query inside envelope often:
-- CREATE INDEX user_events_envelope ON user_events USING GIN (envelope);


-- CREATE TABLE user_events (
--   event_id    text GENERATED ALWAYS AS (envelope->>'event_id') STORED
--   event_type    text GENERATED ALWAYS AS (envelope->>'aggregate_id') STORED
--   envelope       jsonb NOT NULL,
--   event_type    text GENERATED ALWAYS AS (envelope->>'data'->>'type') STORED
-- );
--

    -- if let Ok(row) = client
    --     .query_one(
    --         "SELECT event_id, aggegate_id, event_name, envelope from user_events limit 1;",
    --         &[],
    --     )
    --     .await
    -- {
    --     // let id: Uuid = row.get("id");
    --     // let name: String = row.get("name");
    --     let event_id: String = row.get("event_id");
    --     let aggregate_id: String = row.get("aggregate_id");
    --     let event_name: String = row.get("event_name");
    --     let data: Json<Envelope> = row.get("envelope");
    --     let data: Envelope = data.0;
    --     println!("-->> event_id: {event_id:#?}, event_name: {event_name:#?}, aggregate_id: {aggregate_id:#?} data: {data:#?}");
    -- }

CREATE TABLE states (
  aggregate_id  uuid GENERATED ALWAYS AS ((state->>'aggregate_id')::uuid) STORED PRIMARY KEY,
    -- Optimistic concurrency control
  occ_version   bigint GENERATED ALWAYS AS ((state->>'occ_version')::bigint) STORED,
  timestamp     timestamptz NOT NULL DEFAULT now(),
  state         jsonb NOT NULL
)
