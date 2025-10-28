use crate::{error::Result, user::UserProfileResponse, utils::Sealed};
pub mod non_blocking {
    use super::*;

    #[async_trait::async_trait]
    pub trait User: Sealed {
        async fn get_user_profile(&self) -> Result<UserProfileResponse>;
    }
}
pub mod blocking {
    use super::*;
    pub trait User: Sealed {
        fn get_user_profile(&self) -> Result<UserProfileResponse>;
    }
}
