-- Your SQL goes here
ALTER TABLE tasks ADD COLUMN generation_params text NOT NULL AFTER params;
