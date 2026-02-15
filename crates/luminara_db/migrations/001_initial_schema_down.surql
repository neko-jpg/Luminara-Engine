USE DB project;
REMOVE TABLE project_settings;

USE DB assets;
REMOVE TABLE asset;
REMOVE TABLE depends_on;

USE DB scenes;
REMOVE TABLE scene;
REMOVE TABLE entity;
REMOVE TABLE parent_of;
REMOVE TABLE component;

USE DB history;
REMOVE TABLE undo_entry;

USE DB saves;
REMOVE TABLE save_slot;
