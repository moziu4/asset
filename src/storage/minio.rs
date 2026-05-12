use aws_sdk_s3::{Client, config::Region};
use aws_sdk_s3::primitives::ByteStream;

#[derive(Clone)]
pub struct Storage {
    pub client: Client,
    _bucket: String,
}

impl Storage {
    pub async fn new(
        endpoint: String,
        access_key: String,
        secret_key: String,
        bucket: String,
    ) -> Self {
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .endpoint_url(&endpoint)
            .region(Region::new("us-east-1"))
            .credentials_provider(
                aws_sdk_s3::config::Credentials::new(
                    access_key,
                    secret_key,
                    None,
                    None,
                    "minio",
                ),
            )
            .load()
            .await;

        let s3_config = aws_sdk_s3::config::Builder::from(&config)
            .force_path_style(true)
            .build();

        Self {
            client: Client::from_conf(s3_config),
            _bucket: bucket,
        }
    }

    pub async fn upload_to_bucket(
        &self,
        bucket: &str,
        key: String,
        data: bytes::Bytes,
        content_type: &str,
    ) -> anyhow::Result<()> {
        match self.client
            .put_object()
            .bucket(bucket)
            .key(key)
            .body(ByteStream::from(data))
            .content_type(content_type)
            .send()
            .await {
                Ok(_) => Ok(()),
                Err(e) => {
                    eprintln!("Error uploading to MinIO: {:?}", e);
                    Err(anyhow::anyhow!("MinIO upload failed: {}", e))
                }
            }
    }

    pub async fn ensure_bucket_exists(&self, bucket: &str) -> anyhow::Result<()> {
        let exists = self.client.head_bucket().bucket(bucket).send().await.is_ok();
        
        if !exists {
            println!("Bucket {} does not exist, creating...", bucket);
            self.client.create_bucket().bucket(bucket).send().await?;
        }
        Ok(())
    }

    pub async fn delete_from_bucket(
        &self,
        bucket: &str,
        key: &str,
    ) -> anyhow::Result<()> {
        self.client
            .delete_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await?;
        Ok(())
    }
}
