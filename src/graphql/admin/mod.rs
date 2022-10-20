use crate::graphql::admin::inputs::{AddRoleInput, RemoveRoleInput};
use crate::graphql::guards::RoleGuard;
use crate::graphql::roles::Role;
use crate::models::user::UserEntity;

use async_graphql::{Context, Error, Object, Result};
use mongodb::bson::doc;
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use crate::ModelFor;

pub mod inputs;
pub mod objects;

#[derive(Default)]
pub struct AdminQueries;

#[derive(Default)]
pub struct AdminMutations;

#[derive(Default)]
pub struct AdminSubscriptions;

#[Object]
impl AdminMutations {
    #[graphql(guard = "RoleGuard::new(Role::Admin)")]
    pub async fn add_role(&self, ctx: &Context<'_>, args: AddRoleInput) -> Result<bool> {
        let user = ctx.data::<UserEntity>().unwrap();
        let users = ctx.data::<ModelFor<UserEntity>>().unwrap();

        let filter = doc! { "$or": [{ "_id": args.name_or_id.clone() }, { "name": args.name_or_id.clone() }] };
        let update = doc! { "$addToSet": { "roles": args.role.as_str().clone() }};

        if let Ok(Some(old_record)) = users.find_one(filter.clone(), None).await {
            // Check if the current user has a higher role than the target user
            if !user.roles.iter().any(|r| r < &args.role) {
                return Err(Error::new(format!(
                    "You are not allowed to add role '{}' to user '{}'",
                    args.role.as_str(),
                    args.name_or_id.clone()
                )));
            }
            if old_record.roles.contains(&args.role) {
                return Err(Error::new(format!(
                    "User '{}' already possesses role '{}'",
                    args.name_or_id.clone(),
                    args.role.as_str()
                )));
            }
            let options = FindOneAndUpdateOptions::builder()
                .return_document(ReturnDocument::After)
                .build();
            if let Ok(Some(_)) = users
                .find_one_and_update(filter, update, options)
                .await
            {
                return Ok(true);
            }
        }

        Err(Error::new(format!(
            "Cannot grant role '{}' to user '{}'.",
            args.role.as_str(),
            args.name_or_id
        )))
    }

    #[graphql(guard = "RoleGuard::new(Role::Admin)")]
    pub async fn remove_role(&self, ctx: &Context<'_>, args: RemoveRoleInput) -> Result<bool> {
        let user = ctx.data::<UserEntity>().unwrap();
        let users = ctx.data::<ModelFor<UserEntity>>().unwrap();

        let filter = doc! { "$or": [{ "_id": args.name_or_id.clone() }, { "name": args.name_or_id.clone() }] };
        let update = doc! { "$pull": { "roles": args.role.as_str().clone() }};

        if let Ok(Some(old_record)) = users.find_one(filter.clone(), None).await {
            // Check if the current user has a higher role than the target user
            if !user.roles.iter().any(|r| r < &args.role) {
                return Err(Error::new(format!(
                    "You are not allowed to remove role '{}' from user '{}'",
                    args.role.as_str(),
                    args.name_or_id.clone()
                )));
            }
            if !old_record.roles.contains(&args.role) {
                return Err(Error::new(format!(
                    "User '{}' does not possess role '{}'",
                    args.name_or_id.clone(),
                    args.role.as_str()
                )));
            }
            let options = FindOneAndUpdateOptions::builder()
                .return_document(ReturnDocument::After)
                .build();
            if let Ok(Some(_)) = users
                .find_one_and_update(filter, update, options)
                .await
            {
                return Ok(true);
            }
        }

        Err(Error::new(format!(
            "Cannot remove role '{}' from user '{}'.",
            args.role.as_str(),
            args.name_or_id
        )))
    }
}
