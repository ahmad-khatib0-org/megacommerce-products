use std::sync::Arc;

use dashmap::DashMap;
use megacommerce_proto::{
  Category, CategoryTranslations, ProductDataResponseSubcategory, Subcategory,
  SubcategoryTranslations,
};
use megacommerce_shared::{models::errors::BoxedErr, store::errors::handle_db_error};
use serde_json::from_value;
use sqlx::query;

use crate::store::cache::Cache;

impl Cache {
  pub fn category_data(&self, category_name: &str) -> Option<Arc<Category>> {
    self.categories.get(category_name).map(|cat| cat.value().clone())
  }

  pub fn subcategory_data(
    &self,
    category_name: &str,
    subcategory_name: &str,
    language: &str,
  ) -> Option<ProductDataResponseSubcategory> {
    let sub_guard = self.subcategories_data.get(category_name)?;
    let sub_arc = sub_guard.get(subcategory_name)?.value().clone();

    let lang_guard = self.subcategories_translation.get(category_name)?;
    let subs_map_guard = lang_guard.get(language)?;
    let trans_arc = subs_map_guard.get(subcategory_name)?.value().clone();

    Some(ProductDataResponseSubcategory {
      data: Some((*sub_arc).clone()),
      translations: Some((*trans_arc).clone()),
    })
  }

  pub(super) async fn categories_init(&self) -> Result<(), BoxedErr> {
    let rows = query!("SELECT id, name, image, subcategories, translations FROM categories")
      .fetch_all(self.db.as_ref())
      .await
      .map_err(|err| handle_db_error(err, "products.store.categories_init"))?;

    // clear existing maps
    self.categories.clear();
    self.subcategories_data.clear();
    self.subcategories_translation.clear();

    for c in rows {
      println!("category id: {}", c.id);
      let subcategories: Vec<Subcategory> = from_value(c.subcategories)?; // error handling omitted

      println!("after subcategories");
      let translations: Vec<CategoryTranslations> = from_value(c.translations)?;
      println!("after translations");

      // insert category
      let cat = Arc::new(Category {
        id: c.id.clone(),
        name: c.name.clone(),
        image: c.image.clone(),
        translations: translations.clone(),
        subcategories: subcategories.clone(),
      });
      self.categories.insert(c.id.clone(), cat);

      // build inner dashmap for subcategories
      let inner_sub = DashMap::new();
      for s in &subcategories {
        inner_sub.insert(s.id.clone(), Arc::new(s.clone()));
      }
      self.subcategories_data.insert(c.id.clone(), inner_sub);

      // build translations: language -> ( sub_id -> Arc<SubcategoryTranslations> )
      let langs_map = DashMap::new();
      for tr in &translations {
        let subs_map = DashMap::new();
        for (sub_id, sub_tr) in &tr.subcategories {
          subs_map.insert(
            sub_id.clone(),
            Arc::new(SubcategoryTranslations {
              name: sub_tr.name.clone(),
              attributes: sub_tr.attributes.clone(),
              data: sub_tr.data.clone(),
              safety: sub_tr.safety.clone(),
            }),
          );
        }
        langs_map.insert(tr.language.clone(), subs_map);
      }
      self.subcategories_translation.insert(c.id.clone(), langs_map);
    }

    Ok(())
  }
}
