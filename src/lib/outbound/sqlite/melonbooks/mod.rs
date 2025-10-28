use crate::domain::melonbooks::models::artist::{Artist, ArtistArgs, FollowArtistError, GetArtistsError, UnfollowArtistError};
use crate::domain::melonbooks::models::availability::Availability;
use crate::domain::melonbooks::models::product::{AddSkippingUrlError, AddTitleSkipSequenceError, CreateProductArgs, CreateProductError, DeleteTitleSkipSequenceError, GetProductsError, GetSkippingUrlsError, GetTitleSkipSequencesError, Product, UpdateProductArgs, UpdateProductError};
use crate::domain::melonbooks::ports::MelonbooksRepository;
use crate::outbound::sqlite::melonbooks::models::{ArtistRow, ArtistRowInsert, CategoryRow, CategoryRowInsert, FlagRow, FlagRowInsert, ProductRow, ProductRowInsert, SkipProductArtistRowInsert, SkipProductRow, SkipProductRowInsert, TagRow, TagRowInsert, TitleSkipSequenceRow, TitleSkipSequenceRowInsert};
use crate::outbound::sqlite::{schema, Sqlite};
use anyhow::Context;
use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::sql_types::Text;
use r2d2::PooledConnection;
use schema::melonbooks_artist::dsl as artist_dsl;
use schema::melonbooks_category::dsl as category_dsl;
use schema::melonbooks_flag::dsl as flag_dsl;
use schema::melonbooks_product::dsl as product_dsl;
use schema::melonbooks_product_artist::dsl as product_artist_dsl;
use schema::melonbooks_product_flag::dsl as product_flag_dsl;
use schema::melonbooks_product_tag::dsl as product_tag_dsl;
use schema::melonbooks_skip_product::dsl as skip_product_dsl;
use schema::melonbooks_skip_product_artist::dsl as skip_product_artist_dsl;
use schema::melonbooks_tag::dsl as tag_dsl;
use schema::melonbooks_title_skip_sequence::dsl as title_skip_dsl;

mod models;

impl Sqlite {
    fn get_artist_row_by_id(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
        artist_id: i32
    ) -> Result<Option<ArtistRow>, anyhow::Error> {
        let artist = artist_dsl::melonbooks_artist
            .select(ArtistRow::as_select())
            .filter(artist_dsl::id.eq(artist_id))
            .first(connection)
            .optional()
            .with_context(|| format!("cannot get artist with id '{}'", artist_id))?;
        Ok(artist)
    }

    fn get_artist_row_by_name(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
        artist_name: &str
    ) -> Result<Option<ArtistRow>, anyhow::Error> {
        let artist = artist_dsl::melonbooks_artist
            .select(ArtistRow::as_select())
            .filter(artist_dsl::name.eq(artist_name))
            .first(connection)
            .optional()
            .with_context(|| format!("cannot get artist with name '{}'", artist_name))?;
        Ok(artist)
    }

    fn insert_artist_row(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
        artist_args: &ArtistArgs,
        following: bool,
        date_followed: Option<DateTime<Utc>>,
    ) -> Result<ArtistRow, anyhow::Error> {
        let artist = diesel::insert_into(artist_dsl::melonbooks_artist)
            .values(ArtistRowInsert { name: artist_args.name(), following, date_followed: date_followed.map(|d| d.naive_utc()) })
            .returning(ArtistRow::as_returning())
            .get_result(connection)
            .with_context(|| format!("cannot insert artist with name '{}'", artist_args.name()))?;
        Ok(artist)
    }
    
    fn insert_product_artist_row(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
        product_row: &ProductRow,
        artist_row: &ArtistRow,
    ) -> Result<(), anyhow::Error> {
        diesel::insert_into(product_artist_dsl::melonbooks_product_artist)
            .values((product_artist_dsl::product_id.eq(product_row.id), product_artist_dsl::artist_id.eq(artist_row.id)))
            .execute(connection)
            .with_context(|| format!("cannot insert artist with name '{}' for product '{}'", artist_row.name, product_row.url))?;
        Ok(())
    }

    fn update_artist_row_follow(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
        artist: &ArtistRow,
    ) -> Result<ArtistRow, anyhow::Error> {
        let artist = diesel::update(&artist)
            .set((artist_dsl::following.eq(true), artist_dsl::date_followed.eq(Some(Utc::now().naive_utc()))))
            .returning(ArtistRow::as_returning())
            .get_result(connection)
            .with_context(|| format!("cannot update artist with name '{}' to followed", artist.name))?;
        Ok(artist)
    }

    fn update_artist_row_unfollow(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
        artist: &ArtistRow,
    ) -> Result<ArtistRow, anyhow::Error> {
        let artist = diesel::update(&artist)
            .set((artist_dsl::following.eq(false), artist_dsl::date_followed.eq(Option::<NaiveDateTime>::None)))
            .returning(ArtistRow::as_returning())
            .get_result(connection)
            .with_context(|| format!("cannot update artist with name '{}' to unfollowed", artist.name))?;
        Ok(artist)
    }

    fn get_artist_rows(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<ArtistRow>, anyhow::Error> {
        let artists = artist_dsl::melonbooks_artist
            .select(ArtistRow::as_select())
            .get_results(connection)
            .with_context(|| "cannot select artists")?;
        Ok(artists)
    }

    fn get_product_artists(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
        product: &ProductRow
    ) -> Result<Vec<ArtistRow>, anyhow::Error> {
        let artists = product_artist_dsl::melonbooks_product_artist
            .inner_join(product_dsl::melonbooks_product)
            .inner_join(artist_dsl::melonbooks_artist)
            .select(ArtistRow::as_select())
            .filter(product_dsl::id.eq(product.id))
            .get_results(connection)
            .with_context(|| format!("cannot get artists for product with url '{}'", product.url))?;
        Ok(artists)
    }

    fn get_product_row_by_url(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
        url: &str
    ) -> Result<Option<ProductRow>, anyhow::Error> {
        let product = product_dsl::melonbooks_product
            .select(ProductRow::as_select())
            .filter(product_dsl::url.eq(url))
            .first(connection)
            .optional()
            .with_context(|| format!("cannot get product with url '{}'", url))?;
        Ok(product)
    }

    fn get_product_rows(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<ProductRow>, anyhow::Error> {
        let products = product_dsl::melonbooks_product
            .select(ProductRow::as_select())
            .order_by(product_dsl::date_added.desc())
            .get_results(connection)
            .with_context(|| "cannot get products")?;
        Ok(products)
    }

    fn get_product_rows_by_artist(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
        artist_id: i32
    ) -> Result<Vec<ProductRow>, anyhow::Error> {
        let products = product_artist_dsl::melonbooks_product_artist
            .inner_join(product_dsl::melonbooks_product)
            .select(ProductRow::as_select())
            .filter(product_artist_dsl::artist_id.eq(artist_id))
            .order_by(product_dsl::date_added.desc())
            .get_results(connection)
            .with_context(|| format!("cannot get products by artist with id {}", artist_id))?;
        Ok(products)
    }

    fn insert_product_row(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
        product_args: &CreateProductArgs,
        category: &CategoryRow,
    ) -> Result<ProductRow, anyhow::Error> {
        let product = diesel::insert_into(product_dsl::melonbooks_product)
            .values(ProductRowInsert {
                url: product_args.url(),
                title: product_args.title(),
                circle: product_args.circle(),
                image_url: product_args.image_url(),
                category_id: category.id,
                price: product_args.price(),
                availability: product_args.availability().clone()
            })
            .returning(ProductRow::as_returning())
            .get_result(connection)
            .with_context(|| format!("cannot insert product with url '{}'", product_args.url()))?;
        Ok(product)
    }
    
    fn get_category_by_name(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
        category_name: &str
    ) -> Result<Option<CategoryRow>, anyhow::Error> {
        let category = category_dsl::melonbooks_category
            .filter(category_dsl::category.eq(category_name))
            .first(connection)
            .optional()
            .with_context(|| format!("cannot find category with name '{}'", category_name))?;
        Ok(category)
    }

    fn insert_category_row(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
        category: &str,
    ) -> Result<CategoryRow, anyhow::Error> {
        let category_row = self.get_category_by_name(connection, category)?;
        match category_row {
            Some(category) => Ok(category),
            None => {
                let category = diesel::insert_into(category_dsl::melonbooks_category)
                    .values(CategoryRowInsert { category })
                    .returning(CategoryRow::as_returning())
                    .get_result(connection)
                    .with_context(|| format!("cannot insert category with name '{}'", category))?;
                Ok(category)
            }
        }
    }

    fn get_product_category(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
        product: &ProductRow,
    ) -> Result<CategoryRow, anyhow::Error> {
        let category = category_dsl::melonbooks_category
            .find(product.category_id)
            .first(connection)
            .with_context(|| format!("cannot find category with id '{}'", product.category_id))?;
        Ok(category)
    }

    fn insert_product_tag(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
        product: &ProductRow,
        tag_name: &str,
    ) -> Result<TagRow, anyhow::Error> {
        let tag_row = tag_dsl::melonbooks_tag
            .filter(tag_dsl::tag.eq(tag_name))
            .first(connection)
            .optional()
            .with_context(|| format!("cannot find tag with name '{}'", tag_name))?;
        let tag = match tag_row { 
            Some(tag) => tag,
            None => {
                diesel::insert_into(tag_dsl::melonbooks_tag)
                    .values(TagRowInsert { tag: tag_name })
                    .returning(TagRow::as_returning())
                    .get_result(connection)
                    .with_context(|| format!("cannot insert tag with name '{}'", tag_name))?
            }
        };
        diesel::insert_into(product_tag_dsl::melonbooks_product_tag)
            .values((product_tag_dsl::product_id.eq(product.id), product_tag_dsl::tag_id.eq(tag.id)))
            .execute(connection)
            .with_context(|| format!("cannot insert tag with name '{}' for product '{}'", tag_name, product.url))?;
        Ok(tag)
    }

    fn get_product_tags(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
        product: &ProductRow,
    ) -> Result<Vec<TagRow>, anyhow::Error> {
        let tags = product_tag_dsl::melonbooks_product_tag
            .inner_join(tag_dsl::melonbooks_tag)
            .inner_join(product_dsl::melonbooks_product)
            .select(TagRow::as_select())
            .filter(product_dsl::id.eq(product.id))
            .get_results(connection)
            .with_context(|| format!("cannot get tags for product '{}'", product.url))?;
        Ok(tags)
    }

    fn insert_product_flag(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
        product: &ProductRow,
        flag_name: &str,
    ) -> Result<FlagRow, anyhow::Error> {
        let flag_row = flag_dsl::melonbooks_flag
            .filter(flag_dsl::flag.eq(flag_name))
            .first(connection)
            .optional()
            .with_context(|| format!("cannot find flag with name '{}'", flag_name))?;
        let flag = match flag_row {
            Some(flag) => flag,
            None => {
                diesel::insert_into(flag_dsl::melonbooks_flag)
                    .values(FlagRowInsert { flag: flag_name })
                    .returning(FlagRow::as_returning())
                    .get_result(connection)
                    .with_context(|| format!("cannot insert flag with name '{}'", flag_name))?
            }
        };
        diesel::insert_into(product_flag_dsl::melonbooks_product_flag)
            .values((product_flag_dsl::product_id.eq(product.id), product_flag_dsl::flag_id.eq(flag.id)))
            .execute(connection)
            .with_context(|| format!("cannot insert flag with name '{}' for product '{}'", flag_name, product.url))?;
        Ok(flag)
    }

    fn get_product_flags(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
        product: &ProductRow,
    ) -> Result<Vec<FlagRow>, anyhow::Error> {
        let flags = product_flag_dsl::melonbooks_product_flag
            .inner_join(flag_dsl::melonbooks_flag)
            .inner_join(product_dsl::melonbooks_product)
            .select(FlagRow::as_select())
            .filter(product_dsl::id.eq(product.id))
            .get_results(connection)
            .with_context(|| format!("cannot get flags for product '{}'", product.url))?;
        Ok(flags)
    }

    fn update_product_row(
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
    
    fn add_skip_product<S: AsRef<str>>(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
        url: &str,
        artists: &[S],
    ) -> Result<SkipProductRow, anyhow::Error> {
        let skip_product = diesel::insert_into(skip_product_dsl::melonbooks_skip_product)
            .values(SkipProductRowInsert { url })
            .returning(SkipProductRow::as_returning())
            .get_result(connection)
            .with_context(|| format!("cannot insert skip product with url '{}'", url))?;
        diesel::insert_into(skip_product_artist_dsl::melonbooks_skip_product_artist)
            .values(artists.iter().map(|a| SkipProductArtistRowInsert { skip_product_id: skip_product.id, artist_name: a.as_ref() }).collect::<Vec<_>>())
            .execute(connection)
            .with_context(|| format!("cannot insert artists for skip product with url '{}'", url))?;
        Ok(skip_product)
    }
    
    fn delete_skip_products_for_artist(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
        artist: &str,
    ) -> Result<(), anyhow::Error> {
        diesel::delete(skip_product_dsl::melonbooks_skip_product)
            .filter(skip_product_dsl::id.eq_any(skip_product_artist_dsl::melonbooks_skip_product_artist.filter(skip_product_artist_dsl::artist_name.eq(artist)).select(skip_product_artist_dsl::skip_product_id)))
            .execute(connection)
            .with_context(|| format!("cannot delete skip products for artist '{}'", artist))?;
        Ok(())
    }
    
    fn get_skip_products(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<SkipProductRow>, anyhow::Error> {
        let skip_products = skip_product_dsl::melonbooks_skip_product
            .select(SkipProductRow::as_select())
            .get_results(connection)
            .with_context(|| "cannot get skip products")?;
        Ok(skip_products)
    }
    
    fn add_title_skip_sequence(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
        sequence: &str,
    ) -> Result<TitleSkipSequenceRow, anyhow::Error> {
        let skip_sequence = diesel::insert_into(title_skip_dsl::melonbooks_title_skip_sequence)
            .values(TitleSkipSequenceRowInsert { sequence })
            .returning(TitleSkipSequenceRow::as_returning())
            .get_result(connection)
            .with_context(|| format!("cannot add title skip sequence '{}'", sequence))?;
        Ok(skip_sequence)
    }

    fn delete_title_skip_sequence(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
        sequence: &str,
    ) -> Result<TitleSkipSequenceRow, anyhow::Error> {
        let skip_sequence = diesel::delete(title_skip_dsl::melonbooks_title_skip_sequence)
            .filter(title_skip_dsl::sequence.eq(sequence))
            .returning(TitleSkipSequenceRow::as_returning())
            .get_result(connection)
            .with_context(|| format!("cannot delete title skip sequence '{}'", sequence))?;
        Ok(skip_sequence)
    }
    
    fn get_title_skip_sequences(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<TitleSkipSequenceRow>, anyhow::Error> {
        let title_skip_sequences = title_skip_dsl::melonbooks_title_skip_sequence
            .select(TitleSkipSequenceRow::as_select())
            .get_results(connection)
            .with_context(|| "cannot get title skip sequences")?;
        Ok(title_skip_sequences)
    }

    fn load_product(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
        product: &ProductRow,
    ) -> Result<Product, anyhow::Error> {
        let artists = self.get_product_artists(connection, product)?;
        let tags = self.get_product_tags(connection, product)?;
        let flags = self.get_product_flags(connection, product)?;
        let category = self.get_product_category(connection, product)?;
        let product = Product::new(
            product.id,
            product.date_added.and_utc(),
            product.url.to_owned(),
            product.title.to_owned(),
            product.circle.to_owned(),
            artists.into_iter().map(|a| a.into_domain()).collect(),
            product.image_url.to_owned(),
            category.category,
            tags.into_iter().map(|t| t.into_domain()).collect(),
            flags.into_iter().map(|f| f.into_domain()).collect(),
            product.price.to_owned(),
            product.availability.to_owned(),
        );
        Ok(product)
    }
}


impl MelonbooksRepository for Sqlite {
    async fn follow_melonbooks_artist(&self, args: &ArtistArgs) -> Result<(), FollowArtistError> {
        let mut connection = self.get_connection()?;
        let artist = self.get_artist_row_by_name(&mut connection, args.name())?;
        match artist {
            Some(artist) => {
                if artist.following {
                    return Err(FollowArtistError::AlreadyFollowedError(artist.date_followed.unwrap().and_utc()));
                }
                self.update_artist_row_follow(&mut connection, &artist)?;
            },
            None => {
                self.insert_artist_row(&mut connection, args, true, Some(Utc::now()))?;
            }
        }
        self.delete_skip_products_for_artist(&mut connection, args.name())?;
        Ok(())
    }

    async fn unfollow_melonbooks_artist(&self, artist_id: i32) -> Result<(), UnfollowArtistError> {
        let mut connection = self.get_connection()?;
        let artist = self.get_artist_row_by_id(&mut connection, artist_id)?;
        match artist {
            Some(artist) => {
                if artist.following {
                    self.update_artist_row_unfollow(&mut connection, &artist)?;
                } else {
                    return Err(UnfollowArtistError::ArtistNotFollowed { name: artist.name })
                }
            },
            None => {
                return Err(UnfollowArtistError::UnknownArtist { id: artist_id });
            }
        }
        Ok(())
    }

    async fn get_melonbooks_artists(&self) -> Result<Vec<Artist>, GetArtistsError> {
        let mut connection = self.get_connection()?;
        let artist_rows = self.get_artist_rows(&mut connection)?;
        let artists = artist_rows.into_iter()
            .map(|a| a.into_domain())
            .collect();
        Ok(artists)
    }

    async fn create_melonbooks_product(&self, args: &CreateProductArgs) -> Result<Product, CreateProductError> {
        let mut connection = self.get_connection()?;
        let product_row = self.get_product_row_by_url(&mut connection, args.url())?;
        match product_row {
            Some(product_row) => {
                Err(CreateProductError::DuplicateProduct { url: product_row.url, title: product_row.title })
            },
            None => {
                let product = connection.transaction(|connection| -> Result<Product, anyhow::Error> {
                    let category_row = self.insert_category_row(connection, args.category())?;
                    let product_row = self.insert_product_row(connection, &args, &category_row)?;
                    let mut tags = Vec::new();
                    for tag_name in args.tags() {
                        let tag_row = self.insert_product_tag(connection, &product_row, tag_name)?;
                        tags.push(tag_row);
                    }
                    let mut flags = Vec::new();
                    for flag_name in args.flags() {
                        let flag_row = self.insert_product_flag(connection, &product_row, flag_name)?;
                        flags.push(flag_row);
                    }
                    let mut artists = Vec::new();
                    for artist_name in args.artists() {
                        let artist = self.get_artist_row_by_name(connection, artist_name)?;
                        match artist {
                            Some(artist_row) => {
                                self.insert_product_artist_row(connection, &product_row, &artist_row)?;
                                artists.push(artist_row);
                            },
                            None => {
                                let artist_row = self.insert_artist_row(connection, &ArtistArgs::new(artist_name.to_owned()), false, None)?;
                                self.insert_product_artist_row(connection, &product_row, &artist_row)?;
                                artists.push(artist_row);
                            }
                        }
                    }
                    let product = Product::new(
                        product_row.id,
                        product_row.date_added.and_utc(),
                        product_row.url,
                        product_row.title,
                        product_row.circle,
                        artists.into_iter().map(|a| a.into_domain()).collect(),
                        product_row.image_url,
                        category_row.category,
                        tags.into_iter().map(|t| t.into_domain()).collect(),
                        flags.into_iter().map(|f| f.into_domain()).collect(),
                        product_row.price,
                        product_row.availability
                    );
                    Ok(product)
                })?;
                Ok(product)
            }
        }
    }

    async fn update_melonbooks_product(&self, args: &UpdateProductArgs) -> Result<Product, UpdateProductError> {
        let mut connection = self.get_connection()?;
        let product_row = self.get_product_row_by_url(&mut connection, args.url())?;
        match product_row {
            Some(product_row) => {
                let product_row = self.update_product_row(&mut connection, &product_row, &args)?;
                let product = self.load_product(&mut connection, &product_row)?;
                Ok(product)
            },
            None => Err(UpdateProductError::ProductMissing { url: args.url().to_owned() }),
        }
    }

    async fn get_melonbooks_products(&self) -> Result<Vec<Product>, GetProductsError> {
        let mut connection = self.get_connection()?;
        let product_rows = self.get_product_rows(&mut connection)?;
        let mut products = Vec::new();
        for product_row in product_rows {
            let product = self.load_product(&mut connection, &product_row)?;
            products.push(product);
        }
        Ok(products)
    }

    async fn get_melonbooks_products_by_artist(&self, artist_id: i32) -> Result<Vec<Product>, GetProductsError> {
        let mut connection = self.get_connection()?;
        let product_rows = self.get_product_rows_by_artist(&mut connection, artist_id)?;
        let mut products = Vec::new();
        for product_row in product_rows {
            let product = self.load_product(&mut connection, &product_row)?;
            products.push(product);
        }
        Ok(products)
    }

    async fn add_melonbooks_skipping_url<S: AsRef<str>>(&self, url: &str, artists: &[S]) -> Result<(), AddSkippingUrlError> {
        let mut connection = self.get_connection()?;
        self.add_skip_product(&mut connection, url, artists)?;
        Ok(())
    }

    async fn get_melonbooks_skipping_urls(&self) -> Result<Vec<String>, GetSkippingUrlsError> {
        let mut connection = self.get_connection()?;
        let skip_products = self.get_skip_products(&mut connection)?;
        let urls = skip_products.into_iter()
            .map(|product| product.url)
            .collect();
        Ok(urls)
    }

    async fn add_melonbooks_title_skip_sequence(&self, sequence: &str) -> Result<(), AddTitleSkipSequenceError> {
        let mut connection = self.get_connection()?;
        self.add_title_skip_sequence(&mut connection, sequence)?;
        Ok(())
    }

    async fn delete_melonbooks_title_skip_sequence(&self, sequence: &str) -> Result<(), DeleteTitleSkipSequenceError> {
        let mut connection = self.get_connection()?;
        self.delete_title_skip_sequence(&mut connection, sequence)?;
        Ok(())
    }

    async fn get_melonbooks_title_skip_sequences(&self) -> Result<Vec<String>, GetTitleSkipSequencesError> {
        let mut connection = self.get_connection()?;
        let sequence_rows = self.get_title_skip_sequences(&mut connection)?;
        let sequences = sequence_rows.into_iter().map(|s| s.sequence).collect();
        Ok(sequences)
    }
}

impl Expression for Availability {
    type SqlType = Text;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::domain::melonbooks::models::availability::Availability;

    #[tokio::test]
    async fn test_follow_melonbooks_artist() {
        let db = Sqlite::new_in_memory();
        db.setup().unwrap();
        db.follow_melonbooks_artist(&artist_args()).await.unwrap();

        let artists = db.get_melonbooks_artists().await.unwrap();
        assert_eq!(artists.len(), 1);
        let artist = artists.get(0).unwrap();
        assert_eq!(artist.name(), artist_args().name());
        assert_eq!(artist.following(), true);
        assert_ne!(artist.date_followed(), None);
    }

    #[tokio::test]
    async fn test_unfollow_melonbooks_artist() {
        let db = Sqlite::new_in_memory();
        db.setup().unwrap();
        db.follow_melonbooks_artist(&artist_args()).await.unwrap();
        let artist = db.get_melonbooks_artists().await.unwrap().into_iter().find(|a| a.name().eq(artist_args().name())).unwrap();
        db.unfollow_melonbooks_artist(artist.id()).await.unwrap();

        let artists = db.get_melonbooks_artists().await.unwrap();
        assert_eq!(artists.len(), 1);
        let artist = artists.get(0).unwrap();
        assert_eq!(artist.name(), artist_args().name());
        assert_eq!(artist.following(), false);
        assert_eq!(artist.date_followed(), None);
    }

    #[tokio::test]
    async fn test_get_melonbooks_artists() {
        let db = Sqlite::new_in_memory();
        db.setup().unwrap();
        db.follow_melonbooks_artist(&artist_args()).await.unwrap();
        db.follow_melonbooks_artist(&artist_args2()).await.unwrap();
        let artist2 = db.get_melonbooks_artists().await.unwrap().into_iter().find(|a| a.name().eq(artist_args2().name())).unwrap();
        db.unfollow_melonbooks_artist(artist2.id()).await.unwrap();

        let artists = db.get_melonbooks_artists().await.unwrap();
        assert_eq!(artists.len(), 2);
        assert!(artists.iter().find(|a| a.name().eq(artist_args().name())).is_some());
        assert!(artists.iter().find(|a| a.name().eq(artist_args2().name())).is_some());
    }

    #[tokio::test]
    async fn test_create_melonbooks_product() {
        let db = Sqlite::new_in_memory();
        db.setup().unwrap();
        let args = product_args();
        let product = db.create_melonbooks_product(&args).await.unwrap();

        assert_eq!(product.url(), args.url());
        assert_eq!(product.title(), args.title());
        assert_eq!(product.circle(), args.circle());
        assert_eq!(product.artists().len(), args.artists().len());
        assert_eq!(product.image_url(), args.image_url());
        assert_eq!(product.category(), args.category());
        assert_eq!(product.tags().len(), args.tags().len());
        assert_eq!(product.flags().len(), args.flags().len());
        assert_eq!(product.availability(), args.availability());
    }

    #[tokio::test]
    async fn test_create_melonbooks_product_with_existing_artist() {
        let db = Sqlite::new_in_memory();
        db.setup().unwrap();
        db.follow_melonbooks_artist(&artist_args()).await.unwrap();
        let args = product_args();
        let artists = db.get_melonbooks_artists().await.unwrap();
        let product = db.create_melonbooks_product(&args).await.unwrap();

        assert_eq!(artists.len(), 1);
        assert_eq!(product.artists().len(), args.artists().len());
        assert_eq!(product.artists().get(0).unwrap().id(), artists.get(0).unwrap().id());
    }

    #[tokio::test]
    async fn test_create_melonbooks_product_fails_on_duplicate() {
        let db = Sqlite::new_in_memory();
        db.setup().unwrap();
        let args = product_args();
        let product = db.create_melonbooks_product(&args).await.unwrap();
        let error = db.create_melonbooks_product(&args).await.unwrap_err();
        assert!(matches!(error, CreateProductError::DuplicateProduct { .. }));
    }

    #[tokio::test]
    async fn test_update_melonbooks_product() {
        let db = Sqlite::new_in_memory();
        db.setup().unwrap();
        let args = product_args();
        let product = db.create_melonbooks_product(&args).await.unwrap();
        assert_eq!(product.availability(), Availability::Available);

        let update_args = UpdateProductArgs::new(product.url().to_owned(), Availability::NotAvailable);
        db.update_melonbooks_product(&update_args).await.unwrap();

        let products = db.get_melonbooks_products().await.unwrap();
        assert_eq!(products.len(), 1);
        assert_eq!(products.get(0).unwrap().id(), product.id());
        assert_eq!(products.get(0).unwrap().availability(), Availability::NotAvailable);
    }

    #[tokio::test]
    async fn test_get_melonbooks_products() {
        let db = Sqlite::new_in_memory();
        db.setup().unwrap();
        let args1 = product_args();
        let args2 = product_args2();
        let product1 = db.create_melonbooks_product(&args1).await.unwrap();
        let product2 = db.create_melonbooks_product(&args2).await.unwrap();

        let products = db.get_melonbooks_products().await.unwrap();
        assert_eq!(products.len(), 2);
        assert!(products.iter().find(|p| p.id().eq(&product1.id())).is_some());
        assert!(products.iter().find(|p| p.id().eq(&product2.id())).is_some());
    }

    #[tokio::test]
    async fn test_get_melonbooks_products_by_artist() {
        let db = Sqlite::new_in_memory();
        db.setup().unwrap();
        let args1 = product_args();
        let args2 = product_args2();
        let product1 = db.create_melonbooks_product(&args1).await.unwrap();
        let product2 = db.create_melonbooks_product(&args2).await.unwrap();
        let artists = db.get_melonbooks_artists().await.unwrap();
        let artist1 = artists.iter().find(|a| a.name().eq(artist_args().name())).unwrap();
        let artist2 = artists.iter().find(|a| a.name().eq(artist_args2().name())).unwrap();

        let products = db.get_melonbooks_products_by_artist(artist1.id()).await.unwrap();
        assert_eq!(products.len(), 2);
        assert!(products.iter().filter(|p| p.id().eq(&product1.id())).next().is_some());
        assert!(products.iter().filter(|p| p.id().eq(&product2.id())).next().is_some());

        let products = db.get_melonbooks_products_by_artist(artist2.id()).await.unwrap();
        assert_eq!(products.len(), 1);
        assert!(products.iter().filter(|p| p.id().eq(&product1.id())).next().is_none());
        assert!(products.iter().filter(|p| p.id().eq(&product2.id())).next().is_some());
    }
    
    #[tokio::test]
    async fn test_skip_products() {
        let db = Sqlite::new_in_memory();
        db.setup().unwrap();
        
        db.add_melonbooks_skipping_url(product_args().url(), product_args().artists()).await.unwrap();
        
        let urls = db.get_melonbooks_skipping_urls().await.unwrap();
        assert_eq!(urls.len(), 1);
        assert_eq!(urls.get(0).unwrap(), product_args().url());
    }

    #[tokio::test]
    async fn test_follow_deletes_skip_products() {
        let db = Sqlite::new_in_memory();
        db.setup().unwrap();

        db.add_melonbooks_skipping_url(product_args().url(), product_args().artists()).await.unwrap();

        let urls = db.get_melonbooks_skipping_urls().await.unwrap();
        assert_eq!(urls.len(), 1);
        assert_eq!(urls.get(0).unwrap(), product_args().url());
        
        db.follow_melonbooks_artist(&artist_args()).await.unwrap();
        
        let urls = db.get_melonbooks_skipping_urls().await.unwrap();
        assert_eq!(urls.len(), 0);
    }

    #[tokio::test]
    async fn test_add_title_skip_sequences() {
        let db = Sqlite::new_in_memory();
        db.setup().unwrap();

        db.add_melonbooks_title_skip_sequence("abc").await.unwrap();
        
        let sequences = db.get_melonbooks_title_skip_sequences().await.unwrap();
        
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences.get(0).unwrap(), "abc");
    }

    #[tokio::test]
    async fn test_delete_title_skip_sequences() {
        let db = Sqlite::new_in_memory();
        db.setup().unwrap();
        db.add_melonbooks_title_skip_sequence("abc").await.unwrap();
        let sequences = db.get_melonbooks_title_skip_sequences().await.unwrap();
        assert_eq!(sequences.len(), 1);
        
        db.delete_melonbooks_title_skip_sequence("abc").await.unwrap();
        let sequences = db.get_melonbooks_title_skip_sequences().await.unwrap();
        assert_eq!(sequences.len(), 0);
    }

    fn artist_args() -> ArtistArgs {
        ArtistArgs::new("mafuyu".to_owned())
    }

    fn artist_args2() -> ArtistArgs {
        ArtistArgs::new("kantoku".to_owned())
    }
    
    fn product_args() -> CreateProductArgs {
        CreateProductArgs::new(
            "https://mafuyu.moe".to_owned(),
            "mafuyu_title".to_owned(),
            Some("mafuyu_circle".to_owned()),
            vec![artist_args().name().to_owned()],
            "https://mafuyu.png".to_owned(),
            "category".to_owned(),
            vec![],
            vec![],
            Some("12.500".to_owned()),
            Availability::Available
        )
    }

    fn product_args2() -> CreateProductArgs {
        CreateProductArgs::new(
            "https://kantoku.moe".to_owned(),
            "kantoku_title".to_owned(),
            None,
            vec![artist_args().name().to_owned(), artist_args2().name().to_owned()],
            "https://kantoku.png".to_owned(),
            "category2".to_owned(),
            vec![],
            vec![],
            None,
            Availability::NotAvailable
        )
    }
}