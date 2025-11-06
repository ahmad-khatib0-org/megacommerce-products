use aws_config::{BehaviorVersion, Region};
use aws_credential_types::Credentials;
use aws_sdk_s3::{config::SharedCredentialsProvider, primitives::ByteStream, Client};
use megacommerce_proto::{Config, ConfigFile};
use megacommerce_shared::models::{
  errors::{BoxedErr, ErrorType, InternalError},
  r_lock::RLock,
};

#[derive(Debug)]
pub struct ObjectStorage {
  client: Client,
  config: RLock<Config>,
  config_file: ConfigFile,
}

impl ObjectStorage {
  pub async fn new(config: RLock<Config>) -> Result<Self, InternalError> {
    let cfg = config.get().await.file.clone().unwrap();
    let cred = Credentials::new(
      cfg.amazon_s3_access_key_id(),
      cfg.amazon_s3_secret_access_key(),
      None,
      None,
      "minio",
    );

    let cred_provider = SharedCredentialsProvider::new(cred);
    let region = cfg.amazon_s3_region().to_string();
    let s3_config = aws_sdk_s3::Config::builder()
      .behavior_version(BehaviorVersion::latest())
      .credentials_provider(cred_provider)
      .region(Region::new(region))
      .endpoint_url(cfg.amazon_s3_endpoint())
      .force_path_style(true)
      .build();

    let client = Client::from_conf(s3_config);
    Self::ensure_bucket(&client, cfg.amazon_s3_bucket()).await?;
    Ok(Self { client, config, config_file: cfg })
  }

  pub async fn upload_file(
    &self,
    key: &str,
    body: Vec<u8>,
    content_type: &str,
  ) -> Result<(), BoxedErr> {
    self
      .client
      .put_object()
      .bucket(self.config_file.amazon_s3_bucket())
      .key(format!("{}{}", self.config_file.amazon_s3_path_prefix(), key))
      .content_type(content_type)
      .body(ByteStream::from(body))
      .send()
      .await?;

    Ok(())
  }

  pub async fn ensure_bucket(client: &Client, bucket: &str) -> Result<(), InternalError> {
    match client.head_bucket().bucket(bucket).send().await {
      Ok(_) => {
        println!("Bucket '{}' already exists", bucket);
        Ok(())
      }
      Err(_) => {
        client.create_bucket().bucket(bucket).send().await.map_err(|err| {
          InternalError::new(
            "products.server.ensure_bucket".to_string(),
            Box::new(err),
            ErrorType::HttpResponseError,
            false,
            "faield to create a bucket".to_string(),
          )
        })?;

        println!("Bucket '{}' created successfully", bucket);
        Ok(())
      }
    }
  }

  pub async fn download_file(&self, key: &str) -> Result<Vec<u8>, BoxedErr> {
    let full_key = format!("{}{}", self.config_file.amazon_s3_path_prefix(), key);
    let response = self
      .client
      .get_object()
      .bucket(self.config_file.amazon_s3_bucket())
      .key(full_key)
      .send()
      .await?;
    let data = response.body.collect().await?.into_bytes();
    Ok(data.to_vec())
  }

  pub async fn delete_file(&self, key: &str) -> Result<(), Box<dyn std::error::Error>> {
    let full_key = format!("{}{}", self.config_file.amazon_s3_path_prefix(), key);

    self
      .client
      .delete_object()
      .bucket(self.config_file.amazon_s3_bucket())
      .key(full_key)
      .send()
      .await?;

    Ok(())
  }

  // Check if file exists
  pub async fn file_exists(&self, key: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let full_key = format!("{}{}", self.config_file.amazon_s3_path_prefix(), key);

    match self
      .client
      .head_object()
      .bucket(self.config_file.amazon_s3_bucket())
      .key(full_key)
      .send()
      .await
    {
      Ok(_) => Ok(true),
      Err(_) => Ok(false),
    }
  }
}
