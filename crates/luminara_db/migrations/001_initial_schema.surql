-- ===== Project Database =====
USE DB project;

DEFINE TABLE project_settings SCHEMAFULL;
DEFINE FIELD key      ON project_settings TYPE string;
DEFINE FIELD value    ON project_settings TYPE object;
DEFINE FIELD category ON project_settings TYPE string;
DEFINE INDEX idx_settings_key ON project_settings FIELDS key UNIQUE;

-- Initial settings
CREATE project_settings SET
    key = 'schema_version',
    value = { version: 1 },
    category = 'system';

CREATE project_settings SET
    key = 'engine_version',
    value = { version: '0.1.0' },
    category = 'system';

-- ===== Assets Database =====
USE DB assets;

DEFINE TABLE asset SCHEMAFULL;
DEFINE FIELD uuid           ON asset TYPE uuid;
DEFINE FIELD path           ON asset TYPE string;
DEFINE FIELD asset_type     ON asset TYPE string;
DEFINE FIELD file_hash      ON asset TYPE string;
DEFINE FIELD processed_hash ON asset TYPE option<string>;
DEFINE FIELD file_size      ON asset TYPE int;
DEFINE FIELD created_at     ON asset TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at     ON asset TYPE datetime DEFAULT time::now();
DEFINE FIELD metadata       ON asset TYPE object;
DEFINE FIELD tags           ON asset TYPE array<string>;

DEFINE INDEX idx_asset_uuid ON asset FIELDS uuid UNIQUE;
DEFINE INDEX idx_asset_path ON asset FIELDS path UNIQUE;
DEFINE INDEX idx_asset_type ON asset FIELDS asset_type;

-- Define analyzer explicitly if missing default
DEFINE ANALYZER simple TOKENIZERS blank,punct FILTERS lowercase;
DEFINE INDEX idx_asset_tags ON asset FIELDS tags SEARCH ANALYZER simple BM25;

DEFINE TABLE depends_on SCHEMAFULL TYPE RELATION FROM asset TO asset;
DEFINE FIELD dependency_type ON depends_on TYPE string;

-- ===== Scenes Database =====
USE DB scenes;

DEFINE TABLE scene SCHEMAFULL;
DEFINE FIELD name        ON scene TYPE string;
DEFINE FIELD description ON scene TYPE option<string>;
DEFINE FIELD version     ON scene TYPE string;
DEFINE FIELD tags        ON scene TYPE array<string>;
DEFINE FIELD settings    ON scene TYPE object;
DEFINE FIELD created_at  ON scene TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at  ON scene TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_scene_name ON scene FIELDS name UNIQUE;

DEFINE TABLE entity SCHEMAFULL;
DEFINE FIELD name    ON entity TYPE string;
DEFINE FIELD scene   ON entity TYPE record<scene>;
DEFINE FIELD enabled ON entity TYPE bool DEFAULT true;
DEFINE FIELD tags    ON entity TYPE array<string>;
DEFINE FIELD layer   ON entity TYPE int DEFAULT 0;
DEFINE FIELD order   ON entity TYPE int DEFAULT 0;
DEFINE INDEX idx_entity_scene ON entity FIELDS scene;

DEFINE TABLE parent_of SCHEMAFULL TYPE RELATION FROM entity TO entity;
DEFINE FIELD order ON parent_of TYPE int DEFAULT 0;

DEFINE TABLE component SCHEMAFULL;
DEFINE FIELD entity         ON component TYPE record<entity>;
DEFINE FIELD component_type ON component TYPE string;
DEFINE FIELD data           ON component TYPE object;
DEFINE FIELD schema_version ON component TYPE int DEFAULT 1;
DEFINE INDEX idx_component_entity ON component FIELDS entity;
DEFINE INDEX idx_component_type   ON component FIELDS component_type;

-- ===== History Database =====
USE DB history;

DEFINE TABLE undo_entry SCHEMAFULL;
DEFINE FIELD group_id      ON undo_entry TYPE string;
DEFINE FIELD sequence      ON undo_entry TYPE int;
DEFINE FIELD command_type  ON undo_entry TYPE string;
DEFINE FIELD description   ON undo_entry TYPE string;
DEFINE FIELD forward_data  ON undo_entry TYPE object;
DEFINE FIELD backward_data ON undo_entry TYPE object;
DEFINE FIELD timestamp     ON undo_entry TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_undo_group ON undo_entry FIELDS group_id, sequence;

-- ===== Saves Database =====
USE DB saves;

DEFINE TABLE save_slot SCHEMAFULL;
DEFINE FIELD name        ON save_slot TYPE string;
DEFINE FIELD description ON save_slot TYPE option<string>;
DEFINE FIELD scene_name  ON save_slot TYPE string;
DEFINE FIELD play_time   ON save_slot TYPE duration;
DEFINE FIELD created_at  ON save_slot TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at  ON save_slot TYPE datetime DEFAULT time::now();
DEFINE FIELD screenshot  ON save_slot TYPE option<bytes>;
DEFINE FIELD custom_data ON save_slot TYPE option<object>;
