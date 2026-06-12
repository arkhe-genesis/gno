-- Tabela ampliada para armazenar CIDs
ALTER TABLE documents ADD COLUMN btfs_cid TEXT;
ALTER TABLE documents ADD COLUMN encrypted BOOLEAN DEFAULT FALSE;
