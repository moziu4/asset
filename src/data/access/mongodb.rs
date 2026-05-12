use mongodb::{Collection, Database};
use crate::core::domain::asset::Asset;
use mongodb::bson::doc;
use futures::stream::TryStreamExt;

#[derive(Clone)]
pub struct AssetRepository {
    collection: Collection<Asset>,
}

impl AssetRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("assets"),
        }
    }

    pub async fn create(&self, asset: &Asset) -> anyhow::Result<()> {
        match self.collection.insert_one(asset).await {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("MongoDB insert_one error: {:?}", e);
                Err(anyhow::anyhow!("Database insertion failed: {}", e))
            }
        }
    }

    pub async fn get_by_id(&self, id: &str) -> anyhow::Result<Option<Asset>> {
        let filter = doc! { "_id": id };
        Ok(self.collection.find_one(filter).await?)
    }

    pub async fn get_by_ids(&self, ids: &[String]) -> anyhow::Result<Vec<Asset>> {
        let filter = doc! { "_id": { "$in": ids } };
        let mut cursor = self.collection.find(filter).await?;
        let mut assets = Vec::new();
        while let Some(asset) = cursor.try_next().await? {
            assets.push(asset);
        }
        Ok(assets)
    }

    pub async fn delete(&self, id: &str) -> anyhow::Result<()> {
        let filter = doc! { "_id": id };
        self.collection.delete_one(filter).await?;
        Ok(())
    }

    pub async fn list_by_tenant(&self, tenant_id: &str, asset_type: Option<&str>) -> anyhow::Result<Vec<Asset>> {
        let mut filter = doc! { "tenant_id": tenant_id };
        if let Some(t) = asset_type {
            filter.insert("type", t);
        }
        
        let mut cursor = self.collection.find(filter).await?;
        let mut assets = Vec::new();
        while let Some(asset) = cursor.try_next().await? {
            assets.push(asset);
        }
        Ok(assets)
    }
}
