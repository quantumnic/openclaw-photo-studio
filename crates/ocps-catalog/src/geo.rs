//! Geographic query functionality for photos with GPS coordinates

use anyhow::Result;

/// Photo with GPS coordinates
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GeoPhoto {
    pub id: String,
    pub file_name: String,
    pub lat: f64,
    pub lon: f64,
    pub rating: u8,
}

impl crate::Catalog {
    /// Get all photos that have GPS coordinates
    pub fn get_photos_with_gps(&self) -> Result<Vec<GeoPhoto>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_name, gps_lat, gps_lon, rating
             FROM photos
             WHERE gps_lat IS NOT NULL AND gps_lon IS NOT NULL"
        )?;

        let photos = stmt
            .query_map([], |row: &rusqlite::Row| {
                Ok(GeoPhoto {
                    id: row.get(0)?,
                    file_name: row.get(1)?,
                    lat: row.get(2)?,
                    lon: row.get(3)?,
                    rating: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, rusqlite::Error>>()?;

        Ok(photos)
    }

    /// Set GPS coordinates for a photo
    pub fn set_gps(&self, photo_id: &str, lat: f64, lon: f64) -> Result<()> {
        self.conn.execute(
            "UPDATE photos SET gps_lat = ?, gps_lon = ? WHERE id = ?",
            rusqlite::params![lat, lon, photo_id],
        )?;
        Ok(())
    }

    /// Get photos within a geographic bounding box
    pub fn get_photos_in_bounds(
        &self,
        lat_min: f64,
        lat_max: f64,
        lon_min: f64,
        lon_max: f64,
    ) -> Result<Vec<GeoPhoto>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_name, gps_lat, gps_lon, rating
             FROM photos
             WHERE gps_lat >= ? AND gps_lat <= ?
               AND gps_lon >= ? AND gps_lon <= ?"
        )?;

        let photos = stmt
            .query_map([lat_min, lat_max, lon_min, lon_max], |row: &rusqlite::Row| {
                Ok(GeoPhoto {
                    id: row.get(0)?,
                    file_name: row.get(1)?,
                    lat: row.get(2)?,
                    lon: row.get(3)?,
                    rating: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, rusqlite::Error>>()?;

        Ok(photos)
    }
}

#[cfg(test)]
mod tests {
    use crate::Catalog;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_photo(catalog: &Catalog, file_name: &str, gps: Option<(f64, f64)>) -> String {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let mut sql = "INSERT INTO photos (id, file_path, file_name, file_size, file_hash, date_imported".to_string();
        let mut values = format!("'{}', '/test/{}', '{}', 1000, 'hash', '{}'", id, file_name, file_name, now);

        if let Some((lat, lon)) = gps {
            sql.push_str(", gps_lat, gps_lon");
            values.push_str(&format!(", {}, {}", lat, lon));
        }

        sql.push_str(") VALUES (");
        sql.push_str(&values);
        sql.push(')');

        catalog.conn.execute(&sql, []).unwrap();
        id
    }

    #[test]
    fn test_get_photos_with_gps() {
        let catalog = Catalog::in_memory().unwrap();

        // Insert photo with GPS
        let id1 = create_test_photo(&catalog, "with_gps.jpg", Some((47.3769, 8.5417)));

        // Insert photo without GPS
        let _id2 = create_test_photo(&catalog, "without_gps.jpg", None);

        // Query photos with GPS
        let photos = catalog.get_photos_with_gps().unwrap();

        assert_eq!(photos.len(), 1);
        assert_eq!(photos[0].id, id1);
        assert_eq!(photos[0].file_name, "with_gps.jpg");
        assert!((photos[0].lat - 47.3769).abs() < 0.001);
        assert!((photos[0].lon - 8.5417).abs() < 0.001);
    }

    #[test]
    fn test_get_photos_without_gps() {
        let catalog = Catalog::in_memory().unwrap();

        // Insert only photos without GPS
        let _id1 = create_test_photo(&catalog, "no_gps1.jpg", None);
        let _id2 = create_test_photo(&catalog, "no_gps2.jpg", None);

        // Query should return empty
        let photos = catalog.get_photos_with_gps().unwrap();
        assert_eq!(photos.len(), 0);
    }

    #[test]
    fn test_set_gps() {
        let catalog = Catalog::in_memory().unwrap();

        // Insert photo without GPS
        let id = create_test_photo(&catalog, "test.jpg", None);

        // Verify no GPS initially
        let photos = catalog.get_photos_with_gps().unwrap();
        assert_eq!(photos.len(), 0);

        // Set GPS coordinates
        catalog.set_gps(&id, 46.9481, 7.4474).unwrap();

        // Verify GPS was set
        let photos = catalog.get_photos_with_gps().unwrap();
        assert_eq!(photos.len(), 1);
        assert_eq!(photos[0].id, id);
        assert!((photos[0].lat - 46.9481).abs() < 0.001);
        assert!((photos[0].lon - 7.4474).abs() < 0.001);
    }

    #[test]
    fn test_get_photos_in_bounds() {
        let catalog = Catalog::in_memory().unwrap();

        // Zurich, Switzerland
        let id1 = create_test_photo(&catalog, "zurich.jpg", Some((47.3769, 8.5417)));

        // Bern, Switzerland
        let id2 = create_test_photo(&catalog, "bern.jpg", Some((46.9481, 7.4474)));

        // Tokyo, Japan (far away)
        let _id3 = create_test_photo(&catalog, "tokyo.jpg", Some((35.6762, 139.6503)));

        // Query for Switzerland region (roughly)
        let photos = catalog.get_photos_in_bounds(46.0, 48.0, 7.0, 9.0).unwrap();

        assert_eq!(photos.len(), 2);

        let ids: Vec<String> = photos.iter().map(|p| p.id.clone()).collect();
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }

    #[test]
    fn test_get_photos_in_bounds_empty() {
        let catalog = Catalog::in_memory().unwrap();

        // Insert photo in Zurich
        let _id = create_test_photo(&catalog, "zurich.jpg", Some((47.3769, 8.5417)));

        // Query for Japan region (should be empty)
        let photos = catalog.get_photos_in_bounds(35.0, 36.0, 139.0, 140.0).unwrap();

        assert_eq!(photos.len(), 0);
    }
}
