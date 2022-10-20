use crate::graphql::roles::Role;
use crate::models::user::UserEntity;
use async_graphql::{Context, Error, Guard, Result};

// General Guard to check if a user is authenticated
pub struct AuthGuard;
#[async_trait::async_trait]
impl Guard for AuthGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        match ctx.data::<UserEntity>() {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::new("You need to be authorized!")),
        }
    }
}

// Guard to check if a user inherits a specific role
pub struct RoleGuard {
    pub role: Role,
}

impl RoleGuard {
    pub fn new(role: Role) -> Self {
        Self { role }
    }
}

#[async_trait::async_trait]
impl Guard for RoleGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        match ctx.data::<UserEntity>() {
            Ok(user) => match user.roles.contains(&self.role) {
                true => Ok(()),
                false => Err(Error::new(format!(
                    "You dont have the required role '{}'.",
                    self.role.as_str()
                ))),
            },
            Err(err) => {
                println!("{:?}", err);
                return Err(Error::new("You need to be authorized!"));
            }
        }
    }
}
