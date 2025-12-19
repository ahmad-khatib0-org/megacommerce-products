use prometheus::{HistogramOpts, HistogramVec, IntCounter, Registry};

/// Prometheus metrics collector for products service
#[derive(Clone, Debug)]
pub struct MetricsCollector {
  pub hero_products_total: IntCounter,
  pub hero_products_errors: IntCounter,
  pub best_selling_products_total: IntCounter,
  pub best_selling_products_errors: IntCounter,
  pub newly_added_products_total: IntCounter,
  pub newly_added_products_errors: IntCounter,
  pub big_discount_products_total: IntCounter,
  pub big_discount_products_errors: IntCounter,
  pub products_list_total: IntCounter,
  pub products_list_errors: IntCounter,
  pub products_category_total: IntCounter,
  pub products_category_errors: IntCounter,
  pub products_to_like_total: IntCounter,
  pub products_to_like_errors: IntCounter,
  pub product_details_total: IntCounter,
  pub product_details_errors: IntCounter,
  pub product_create_total: IntCounter,
  pub product_create_errors: IntCounter,
  pub category_navbar_total: IntCounter,
  pub category_navbar_errors: IntCounter,
  pub product_snapshot_total: IntCounter,
  pub product_snapshot_errors: IntCounter,
  pub product_data_total: IntCounter,
  pub product_data_errors: IntCounter,
  pub cache_hits: IntCounter,
  pub cache_misses: IntCounter,
  pub db_query_duration_seconds: HistogramVec,
  pub request_duration_seconds: HistogramVec,
}

impl MetricsCollector {
  pub fn new(registry: &Registry) -> Result<Self, String> {
    // Hero products
    let hero_products_total =
      IntCounter::new("products_hero_products_total", "Total hero products requests")
        .map_err(|e| e.to_string())?;
    registry.register(Box::new(hero_products_total.clone())).map_err(|e| e.to_string())?;

    let hero_products_errors =
      IntCounter::new("products_hero_products_errors_total", "Total failed hero products requests")
        .map_err(|e| e.to_string())?;
    registry.register(Box::new(hero_products_errors.clone())).map_err(|e| e.to_string())?;

    // Best selling products
    let best_selling_products_total = IntCounter::new(
      "products_best_selling_products_total",
      "Total best selling products requests",
    )
    .map_err(|e| e.to_string())?;
    registry.register(Box::new(best_selling_products_total.clone())).map_err(|e| e.to_string())?;

    let best_selling_products_errors = IntCounter::new(
      "products_best_selling_products_errors_total",
      "Total failed best selling products requests",
    )
    .map_err(|e| e.to_string())?;
    registry.register(Box::new(best_selling_products_errors.clone())).map_err(|e| e.to_string())?;

    // Newly added products
    let newly_added_products_total =
      IntCounter::new("products_newly_added_products_total", "Total newly added products requests")
        .map_err(|e| e.to_string())?;
    registry.register(Box::new(newly_added_products_total.clone())).map_err(|e| e.to_string())?;

    let newly_added_products_errors = IntCounter::new(
      "products_newly_added_products_errors_total",
      "Total failed newly added products requests",
    )
    .map_err(|e| e.to_string())?;
    registry.register(Box::new(newly_added_products_errors.clone())).map_err(|e| e.to_string())?;

    // Big discount products
    let big_discount_products_total = IntCounter::new(
      "products_big_discount_products_total",
      "Total big discount products requests",
    )
    .map_err(|e| e.to_string())?;
    registry.register(Box::new(big_discount_products_total.clone())).map_err(|e| e.to_string())?;

    let big_discount_products_errors = IntCounter::new(
      "products_big_discount_products_errors_total",
      "Total failed big discount products requests",
    )
    .map_err(|e| e.to_string())?;
    registry.register(Box::new(big_discount_products_errors.clone())).map_err(|e| e.to_string())?;

    // Products list
    let products_list_total =
      IntCounter::new("products_products_list_total", "Total products list requests")
        .map_err(|e| e.to_string())?;
    registry.register(Box::new(products_list_total.clone())).map_err(|e| e.to_string())?;

    let products_list_errors =
      IntCounter::new("products_products_list_errors_total", "Total failed products list requests")
        .map_err(|e| e.to_string())?;
    registry.register(Box::new(products_list_errors.clone())).map_err(|e| e.to_string())?;

    // Products category
    let products_category_total =
      IntCounter::new("products_products_category_total", "Total products category requests")
        .map_err(|e| e.to_string())?;
    registry.register(Box::new(products_category_total.clone())).map_err(|e| e.to_string())?;

    let products_category_errors = IntCounter::new(
      "products_products_category_errors_total",
      "Total failed products category requests",
    )
    .map_err(|e| e.to_string())?;
    registry.register(Box::new(products_category_errors.clone())).map_err(|e| e.to_string())?;

    // Products to like
    let products_to_like_total =
      IntCounter::new("products_products_to_like_total", "Total products to like requests")
        .map_err(|e| e.to_string())?;
    registry.register(Box::new(products_to_like_total.clone())).map_err(|e| e.to_string())?;

    let products_to_like_errors = IntCounter::new(
      "products_products_to_like_errors_total",
      "Total failed products to like requests",
    )
    .map_err(|e| e.to_string())?;
    registry.register(Box::new(products_to_like_errors.clone())).map_err(|e| e.to_string())?;

    // Product details
    let product_details_total =
      IntCounter::new("products_product_details_total", "Total product details requests")
        .map_err(|e| e.to_string())?;
    registry.register(Box::new(product_details_total.clone())).map_err(|e| e.to_string())?;

    let product_details_errors = IntCounter::new(
      "products_product_details_errors_total",
      "Total failed product details requests",
    )
    .map_err(|e| e.to_string())?;
    registry.register(Box::new(product_details_errors.clone())).map_err(|e| e.to_string())?;

    // Product create
    let product_create_total =
      IntCounter::new("products_product_create_total", "Total product create requests")
        .map_err(|e| e.to_string())?;
    registry.register(Box::new(product_create_total.clone())).map_err(|e| e.to_string())?;

    let product_create_errors = IntCounter::new(
      "products_product_create_errors_total",
      "Total failed product create requests",
    )
    .map_err(|e| e.to_string())?;
    registry.register(Box::new(product_create_errors.clone())).map_err(|e| e.to_string())?;

    // Category navbar
    let category_navbar_total =
      IntCounter::new("products_category_navbar_total", "Total category navbar requests")
        .map_err(|e| e.to_string())?;
    registry.register(Box::new(category_navbar_total.clone())).map_err(|e| e.to_string())?;

    let category_navbar_errors = IntCounter::new(
      "products_category_navbar_errors_total",
      "Total failed category navbar requests",
    )
    .map_err(|e| e.to_string())?;
    registry.register(Box::new(category_navbar_errors.clone())).map_err(|e| e.to_string())?;

    // Product snapshot
    let product_snapshot_total =
      IntCounter::new("products_product_snapshot_total", "Total product snapshot requests")
        .map_err(|e| e.to_string())?;
    registry.register(Box::new(product_snapshot_total.clone())).map_err(|e| e.to_string())?;

    let product_snapshot_errors = IntCounter::new(
      "products_product_snapshot_errors_total",
      "Total failed product snapshot requests",
    )
    .map_err(|e| e.to_string())?;
    registry.register(Box::new(product_snapshot_errors.clone())).map_err(|e| e.to_string())?;

    // Product data
    let product_data_total =
      IntCounter::new("products_product_data_total", "Total product data requests")
        .map_err(|e| e.to_string())?;
    registry.register(Box::new(product_data_total.clone())).map_err(|e| e.to_string())?;

    let product_data_errors =
      IntCounter::new("products_product_data_errors_total", "Total failed product data requests")
        .map_err(|e| e.to_string())?;
    registry.register(Box::new(product_data_errors.clone())).map_err(|e| e.to_string())?;

    // Cache metrics
    let cache_hits = IntCounter::new("products_cache_hits_total", "Total cache hits")
      .map_err(|e| e.to_string())?;
    registry.register(Box::new(cache_hits.clone())).map_err(|e| e.to_string())?;

    let cache_misses = IntCounter::new("products_cache_misses_total", "Total cache misses")
      .map_err(|e| e.to_string())?;
    registry.register(Box::new(cache_misses.clone())).map_err(|e| e.to_string())?;

    // Latency metrics
    let db_query_duration_seconds = HistogramVec::new(
      HistogramOpts::new(
        "products_db_query_duration_seconds",
        "Database query duration in seconds",
      ),
      &["query"],
    )
    .map_err(|e| e.to_string())?;
    registry.register(Box::new(db_query_duration_seconds.clone())).map_err(|e| e.to_string())?;

    let request_duration_seconds = HistogramVec::new(
      HistogramOpts::new("products_request_duration_seconds", "Request duration in seconds"),
      &[],
    )
    .map_err(|e| e.to_string())?;
    registry.register(Box::new(request_duration_seconds.clone())).map_err(|e| e.to_string())?;

    Ok(MetricsCollector {
      hero_products_total,
      hero_products_errors,
      best_selling_products_total,
      best_selling_products_errors,
      newly_added_products_total,
      newly_added_products_errors,
      big_discount_products_total,
      big_discount_products_errors,
      products_list_total,
      products_list_errors,
      products_category_total,
      products_category_errors,
      products_to_like_total,
      products_to_like_errors,
      product_details_total,
      product_details_errors,
      product_create_total,
      product_create_errors,
      category_navbar_total,
      category_navbar_errors,
      product_snapshot_total,
      product_snapshot_errors,
      product_data_total,
      product_data_errors,
      cache_hits,
      cache_misses,
      db_query_duration_seconds,
      request_duration_seconds,
    })
  }

  pub fn record_hero_products_success(&self, duration_secs: f64) {
    self.hero_products_total.inc();
    self.request_duration_seconds.with_label_values(&[]).observe(duration_secs);
  }

  pub fn record_hero_products_error(&self) {
    self.hero_products_total.inc();
    self.hero_products_errors.inc();
  }

  pub fn record_best_selling_products_success(&self, duration_secs: f64) {
    self.best_selling_products_total.inc();
    self.request_duration_seconds.with_label_values(&[]).observe(duration_secs);
  }

  pub fn record_best_selling_products_error(&self) {
    self.best_selling_products_total.inc();
    self.best_selling_products_errors.inc();
  }

  pub fn record_newly_added_products_success(&self, duration_secs: f64) {
    self.newly_added_products_total.inc();
    self.request_duration_seconds.with_label_values(&[]).observe(duration_secs);
  }

  pub fn record_newly_added_products_error(&self) {
    self.newly_added_products_total.inc();
    self.newly_added_products_errors.inc();
  }

  pub fn record_big_discount_products_success(&self, duration_secs: f64) {
    self.big_discount_products_total.inc();
    self.request_duration_seconds.with_label_values(&[]).observe(duration_secs);
  }

  pub fn record_big_discount_products_error(&self) {
    self.big_discount_products_total.inc();
    self.big_discount_products_errors.inc();
  }

  pub fn record_products_list_success(&self, duration_secs: f64) {
    self.products_list_total.inc();
    self.request_duration_seconds.with_label_values(&[]).observe(duration_secs);
  }

  pub fn record_products_list_error(&self) {
    self.products_list_total.inc();
    self.products_list_errors.inc();
  }

  pub fn record_products_category_success(&self, duration_secs: f64) {
    self.products_category_total.inc();
    self.request_duration_seconds.with_label_values(&[]).observe(duration_secs);
  }

  pub fn record_products_category_error(&self) {
    self.products_category_total.inc();
    self.products_category_errors.inc();
  }

  pub fn record_products_to_like_success(&self, duration_secs: f64) {
    self.products_to_like_total.inc();
    self.request_duration_seconds.with_label_values(&[]).observe(duration_secs);
  }

  pub fn record_products_to_like_error(&self) {
    self.products_to_like_total.inc();
    self.products_to_like_errors.inc();
  }

  pub fn record_product_details_success(&self, duration_secs: f64) {
    self.product_details_total.inc();
    self.request_duration_seconds.with_label_values(&[]).observe(duration_secs);
  }

  pub fn record_product_details_error(&self) {
    self.product_details_total.inc();
    self.product_details_errors.inc();
  }

  pub fn record_product_create_success(&self, duration_secs: f64) {
    self.product_create_total.inc();
    self.request_duration_seconds.with_label_values(&[]).observe(duration_secs);
  }

  pub fn record_product_create_error(&self) {
    self.product_create_total.inc();
    self.product_create_errors.inc();
  }

  pub fn record_category_navbar_success(&self, duration_secs: f64) {
    self.category_navbar_total.inc();
    self.request_duration_seconds.with_label_values(&[]).observe(duration_secs);
  }

  pub fn record_category_navbar_error(&self) {
    self.category_navbar_total.inc();
    self.category_navbar_errors.inc();
  }

  pub fn record_product_snapshot_success(&self, duration_secs: f64) {
    self.product_snapshot_total.inc();
    self.request_duration_seconds.with_label_values(&[]).observe(duration_secs);
  }

  pub fn record_product_snapshot_error(&self) {
    self.product_snapshot_total.inc();
    self.product_snapshot_errors.inc();
  }

  pub fn record_product_data_success(&self, duration_secs: f64) {
    self.product_data_total.inc();
    self.request_duration_seconds.with_label_values(&[]).observe(duration_secs);
  }

  pub fn record_product_data_error(&self) {
    self.product_data_total.inc();
    self.product_data_errors.inc();
  }

  pub fn record_cache_hit(&self) {
    self.cache_hits.inc();
  }

  pub fn record_cache_miss(&self) {
    self.cache_misses.inc();
  }

  pub fn observe_db_query_duration(&self, query_name: &str, duration_secs: f64) {
    self.db_query_duration_seconds.with_label_values(&[query_name]).observe(duration_secs);
  }
}
