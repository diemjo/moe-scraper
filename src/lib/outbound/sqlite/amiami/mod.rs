use crate::domain::amiami::models::product::{CreateProductArgs, CreateProductError, GetCategoriesError, GetProductsError, Product, UpdateProductArgs, UpdateProductError};
use crate::domain::amiami::ports::AmiamiRepository;
use crate::outbound::sqlite::amiami::models::{CategoryRow, CategoryRowInsert, ProductRow, ProductRowInsert};
use crate::outbound::sqlite::schema::amiami_category::dsl as category_dsl;
use crate::outbound::sqlite::schema::amiami_product::dsl as product_dsl;
use crate::outbound::sqlite::Sqlite;
use anyhow::Context;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use r2d2::PooledConnection;

mod models;

impl Sqlite {
    fn get_amiami_product_row_by_url(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
        url: &str
    ) -> Result<Option<ProductRow>, anyhow::Error> {
        let product = product_dsl::amiami_product
            .select(ProductRow::as_select())
            .filter(product_dsl::url.eq(url))
            .first(connection)
            .optional()
            .with_context(|| format!("cannot get product with url '{}'", url))?;
        Ok(product)
    }

    fn get_amiami_product_rows(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<ProductRow>, anyhow::Error> {
        let products = product_dsl::amiami_product
            .select(ProductRow::as_select())
            .order_by(product_dsl::date_added.desc())
            .get_results(connection)
            .with_context(|| "cannot get products")?;
        Ok(products)
    }


    fn insert_amiami_product_row(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
        product_args: &CreateProductArgs,
        category: &CategoryRow,
    ) -> Result<ProductRow, anyhow::Error> {
        let product = diesel::insert_into(product_dsl::amiami_product)
            .values(ProductRowInsert {
                url: product_args.url(),
                title: product_args.title(),
                image_url: product_args.image_url(),
                category_id: category.id,
                maker: product_args.maker(),
                full_price: product_args.full_price(),
                min_price: product_args.min_price(),
                availability: product_args.availability().clone()
            })
            .returning(ProductRow::as_returning())
            .get_result(connection)
            .with_context(|| format!("cannot insert product with url '{}'", product_args.url()))?;
        Ok(product)
    }

    fn update_amiami_product_row(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
        product: &ProductRow,
        args: &UpdateProductArgs,
    ) -> Result<ProductRow, anyhow::Error> {
        let product = diesel::update(&product)
            .set(product_dsl::availability.eq(args.availability().to_string()))
            .returning(ProductRow::as_returning())
            .get_result(connection)
            .with_context(|| format!("cannot update product with url '{}'", product.url))?;
        Ok(product)
    }

    fn get_following_amiami_category_rows(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<CategoryRow>, anyhow::Error> {
        let categories = category_dsl::amiami_category
            .select(CategoryRow::as_select())
            .filter(category_dsl::following.eq(true))
            .get_results(connection)
            .with_context(|| "cannot get categories")?;
        Ok(categories)
    }

    fn get_amiami_category_by_name(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
        category_name: &str
    ) -> Result<Option<CategoryRow>, anyhow::Error> {
        let category = category_dsl::amiami_category
            .filter(category_dsl::category.eq(category_name))
            .first(connection)
            .optional()
            .with_context(|| format!("cannot find category with name '{}'", category_name))?;
        Ok(category)
    }

    fn insert_amiami_category_row(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
        category: &str,
    ) -> Result<CategoryRow, anyhow::Error> {
        let category_row = self.get_amiami_category_by_name(connection, category)?;
        match category_row {
            Some(category) => Ok(category),
            None => {
                let category = diesel::insert_into(category_dsl::amiami_category)
                    .values(CategoryRowInsert { category, following: false })
                    .returning(CategoryRow::as_returning())
                    .get_result(connection)
                    .with_context(|| format!("cannot insert category with name '{}'", category))?;
                Ok(category)
            }
        }
    }

    fn get_amiami_product_category(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
        product: &ProductRow,
    ) -> Result<CategoryRow, anyhow::Error> {
        let category = category_dsl::amiami_category
            .find(product.category_id)
            .first(connection)
            .with_context(|| format!("cannot find category with id '{}'", product.category_id))?;
        Ok(category)
    }

    fn load_amiami_product(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
        product: &ProductRow,
    ) -> Result<Product, anyhow::Error> {
        let category = self.get_amiami_product_category(connection, product)?;
        let product = Product::new(
            product.id,
            product.date_added.and_utc(),
            product.url.to_owned(),
            product.title.to_owned(),
            product.image_url.to_owned(),
            category.category,
            product.maker.to_owned(),
            product.full_price.to_owned(),
            product.min_price.to_owned(),
            product.availability.to_owned(),
        );
        Ok(product)
    }
}

impl AmiamiRepository for Sqlite {

    async fn create_amiami_product(&self, args: &CreateProductArgs) -> Result<Product, CreateProductError> {
        let mut connection = self.get_connection()?;
        let product_row = self.get_amiami_product_row_by_url(&mut connection, args.url())?;
        match product_row {
            Some(product_row) => {
                Err(CreateProductError::DuplicateProduct { url: product_row.url, title: product_row.title })
            },
            None => {
                let product = connection.transaction(|connection| -> Result<Product, anyhow::Error> {
                    let category_row = self.insert_amiami_category_row(connection, args.category())?;
                    let product_row = self.insert_amiami_product_row(connection, &args, &category_row)?;
                    let product = Product::new(
                        product_row.id,
                        product_row.date_added.and_utc(),
                        product_row.url,
                        product_row.title,
                        product_row.image_url,
                        category_row.category,
                        product_row.maker,
                        product_row.full_price,
                        product_row.min_price,
                        product_row.availability
                    );
                    Ok(product)
                })?;
                Ok(product)
            }
        }
    }

    async fn update_amiami_product(&self, args: &UpdateProductArgs) -> Result<Product, UpdateProductError> {
        let mut connection = self.get_connection()?;
        let product_row = self.get_amiami_product_row_by_url(&mut connection, args.url())?;
        match product_row {
            Some(product_row) => {
                let product_row = self.update_amiami_product_row(&mut connection, &product_row, &args)?;
                let product = self.load_amiami_product(&mut connection, &product_row)?;
                Ok(product)
            },
            None => Err(UpdateProductError::ProductMissing { url: args.url().to_owned() }),
        }
    }

    async fn get_amiami_products(&self) -> Result<Vec<Product>, GetProductsError> {
        let mut connection = self.get_connection()?;
        let product_rows = self.get_amiami_product_rows(&mut connection)?;
        let mut products = Vec::new();
        for product_row in product_rows {
            let product = self.load_amiami_product(&mut connection, &product_row)?;
            products.push(product);
        }
        Ok(products)
    }

    async fn get_following_amiami_categories(&self) -> Result<Vec<String>, GetCategoriesError> {
        let mut connection = self.get_connection()?;
        let category_rows = self.get_following_amiami_category_rows(&mut connection)?;
        let categories = category_rows.into_iter()
            .map(|c| c.category)
            .collect();
        Ok(categories)
    }
}

