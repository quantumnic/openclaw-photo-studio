-- Migration 0002: Add IPTC metadata fields to photos table
ALTER TABLE photos ADD COLUMN copyright TEXT;
ALTER TABLE photos ADD COLUMN creator TEXT;
ALTER TABLE photos ADD COLUMN city TEXT;
ALTER TABLE photos ADD COLUMN country TEXT;
